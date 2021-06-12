use std::collections::HashMap;
use std::env::var;

use redis::{Commands, RedisError, Connection, ConnectionLike};
use serde::{Serialize, Deserialize};
use indicatif::{ProgressBar, ProgressIterator};
use crate::render::basic::render_pixel;
use crate::scene::scene::Scene;
use minifb::{Window, WindowOptions, Key};
use std::time::{Instant, Duration};
use std::thread::{sleep, JoinHandle};
use std::thread;
use crossbeam::channel::{Sender, Receiver};

#[derive(Serialize, Deserialize, Debug)]
enum DistributedMessage {
    ProcessPixel(usize, usize),
    SetPixel {
        x: usize,
        y: usize,
        r: u8,
        g: u8,
        b: u8,
    },
}

pub fn run_distributed(commands: &[String], options: &HashMap<String, String>) {
    match commands[0].as_str() {
        "init" => run_init(options),
        "status" => run_status(),
        "reset" => run_reset(),
        "worker" => run_worker(),
        "display" => run_display(),
        other => error!("Unknown distributed command: {:?}", other),
    }
}

//noinspection DuplicatedCode
fn run_init(options: &HashMap<String, String>) {
    info!("Loading scene");
    let scene = sceneformat::read(
        options.get("source").expect("Expected source .cowscene to be set")
    ).expect("Failed to read cowscene file");
    let scene_binary = sceneformat::encode(&scene)
        .expect("Failed to encode .cowscene to binary");

    info!("Creating a new distributed task");
    let (_, mut redis_connection) = connect_to_redis();
    info!("Connected to redis");
    redis_connection.set::<String, Vec<u8>, ()>("turbocow_scene".to_string(), scene_binary)
        .expect("Failed to save turbocow_scene to redis");
    info!("scene saved to redis");

    let (width, height) = match scene.render_options {
        Some(v) => (v.width as usize, v.height as usize),
        None => (1000, 1000)
    };

    let mut tasks = Vec::new();
    for y in 0..height {
        for x in 0..width {
            tasks.push(DistributedMessage::ProcessPixel(x, y));
        }
    }

    let center = (width / 2, height / 2);
    tasks.sort_by(|a, b| {
        let a_dist = match a {
            DistributedMessage::ProcessPixel(x, y) => {
                (center.0 as isize - *x as isize).pow(2) + (center.1 as isize - *y as isize).pow(2)
            },
            other => panic!("Expected process pixel message, got: {:?}", other),
        };
        let b_dist = match b {
            DistributedMessage::ProcessPixel(x, y) => {
                (center.0 as isize - *x as isize).pow(2) + (center.1 as isize - *y as isize).pow(2)
            },
            other => panic!("Expected process pixel message, got: {:?}", other),
        };

        a_dist.partial_cmp(&b_dist).expect("Expected isize comparisong to succeed")
    });

    let mut pipeline = redis::pipe();
    let mut pipeline_ref = &mut pipeline;
    let mut tasks_in_pipeline = 0;
    for i in (0..tasks.len()).progress() {
        let task = &tasks[i];
        let message = bincode::serialize(&task)
                .expect("failed to serialize distributed message");
        pipeline_ref = pipeline_ref.rpush::<String, Vec<u8>>("turbocow_tasks".to_string(), message)
                .ignore();
        tasks_in_pipeline += 1;

        if tasks_in_pipeline >= 10000 || i == tasks.len() - 1 {
            pipeline_ref.query::<()>(&mut redis_connection).expect("failed to send a pipeline query to redis");
            tasks_in_pipeline = 0;
            pipeline = redis::pipe();
            pipeline_ref = &mut pipeline;
        }
    }
}

fn run_worker() {
    run_worker_with_retries(0)
}

fn run_worker_with_retries(retries: usize) {
    if retries > 10 {
        error!("maximum number of retries for worker has reached. Quitting...");
        return;
    }

    info!("connecting to redis...");
    let (_, mut redis_connection) = connect_to_redis();
    info!("connected to redis");

    let result: Vec<u8> = match redis_connection.get("turbocow_scene") {
        Ok(v) => v,
        Err(err) => {
            warn!("failed to get scene from redis, retrying...");
            thread::sleep(Duration::from_secs(1));
            return run_worker_with_retries(retries + 1);
        }
    };
    let scene = sceneformat::decode(&result).expect("Failed to load sceneformat scene");
    let (viewport_width, viewport_height) = match &scene.render_options {
        Some(v) => (v.width as usize, v.height as usize),
        None => (1000, 1000),
    };
    let scene = Scene::from_sceneformat(scene);

    let (task_tx, task_rx) = crossbeam::channel::unbounded();
    let (pixel_tx, pixel_rx) = crossbeam::channel::unbounded();
    let task_io_thread = start_task_io_thread(task_tx.clone(), pixel_rx.clone());

    let mut last_checkpoint = Instant::now();
    let mut total_pixels_rendered = 0;

    loop {
        if task_rx.len() == 0 {
            // wait for new task to appear in queue.
            sleep(Duration::from_millis(16));
            continue;
        }

        let task = task_rx.recv()
            .expect("Failed to read task from queue");

        match task {
            DistributedMessage::ProcessPixel(x, y) => worker_process_pixel(&pixel_tx, &scene, viewport_width, viewport_height, x, y),
            other => panic!("Did not expect this message in tasks queue: {:?}", other),
        }

        total_pixels_rendered += 1;
        let current_time = Instant::now();
        let seconds_passed = (current_time - last_checkpoint).as_secs();
        if seconds_passed >= 10 {
            info!("rendering pixels: {} pixels/second", total_pixels_rendered / seconds_passed);
            last_checkpoint = current_time;
            total_pixels_rendered = 0;
        }
    }
}

fn start_task_io_thread(task_tx: Sender<DistributedMessage>, pixel_rx: Receiver<DistributedMessage>) -> JoinHandle<()> {
    thread::spawn(move || {
        info!("connecting to redis (io thread)...");
        let (_, mut redis_connection) = connect_to_redis();
        info!("connected to redis (io thread)");

        let mut target_queue_size = 8;
        let max_queue_size = 4096;

        loop {
            let tasks_in_queue = task_tx.len();
            let tasks_to_add = if tasks_in_queue < target_queue_size {
                target_queue_size - tasks_in_queue
            } else {
                0
            };

            let mut io_performed = false;
            if tasks_to_add > 0 {
                let mut pipeline = &mut redis::pipe();
                for _ in 0..tasks_to_add {
                    pipeline = pipeline.lpop("turbocow_tasks");
                }
                let result: Vec<Vec<u8>> = pipeline.query(&mut redis_connection)
                    .expect("Failed to get tasks from redis");

                for task in result {
                    if task.len() == 0 {
                        break;
                    }

                    let task: DistributedMessage = bincode::deserialize(&task).expect("Failed to deserialize task");
                    task_tx.send(task).expect("Failed to send task to queue");
                    io_performed = true;
                }

                if io_performed {
                    if tasks_in_queue == 0 {
                        if target_queue_size < max_queue_size {
                            target_queue_size = (target_queue_size * 2).min(max_queue_size);
                            info!("queue does not have enough tasks, setting target size to: {:?}", target_queue_size);
                        }
                    } else if tasks_in_queue > 100 && target_queue_size > 512 {
                        target_queue_size = target_queue_size / 2;
                        info!("queue is overloaded, setting target size to {:?}", target_queue_size);
                    } else if tasks_in_queue > 16 {
                        target_queue_size = (target_queue_size - 1).max(8);
                        info!("queue is overloaded, setting target size to {:?}", target_queue_size);
                    }
                }
            }

            let pixels_in_queue = pixel_rx.len();
            if pixels_in_queue > 0 {
                let mut pipeline = &mut redis::pipe();
                for _ in 0..pixels_in_queue {
                    let pixel = pixel_rx.recv().expect("Failed to read pixel from queue");
                    let pixel_msg = bincode::serialize(&pixel)
                        .expect("Failed to serialize set pixel message");
                    pipeline = pipeline.rpush("turbocow_pixels", pixel_msg).ignore();
                    io_performed = true;
                }
                pipeline.query::<()>(&mut redis_connection)
                    .expect("Failed to send pixels to turbocow_pixels");
            }

            thread::sleep(Duration::from_millis(if io_performed {
                0
            } else {
                16
            }));
        }
    })
}

fn worker_process_pixel(pixel_tx: &Sender<DistributedMessage>, scene: &Scene, viewport_width: usize, viewport_height: usize, x: usize, y: usize) {
    let pixel = render_pixel(scene, viewport_width, viewport_height, x, y);
    pixel_tx.send(DistributedMessage::SetPixel {
        x,
        y,
        r: pixel.red,
        g: pixel.green,
        b: pixel.blue,
    }).expect("Failed to write pixel to send queue");
}

fn run_display() {
    info!("connecting to redis...");
    let (_, mut redis_connection) = connect_to_redis();
    info!("connected to redis");

    let scene: Vec<u8> = redis_connection.get("turbocow_scene").expect("Failed to get scene from redis");
    let scene = sceneformat::decode(&scene).expect("Failed to decode scene");
    let (width, height) = match scene.render_options {
        Some(v) => (v.width as usize, v.height as usize),
        None => (1000, 1000)
    };

    let mut window = Window::new(
        "turbocow",
        width,
        height,
        WindowOptions::default()
    ).expect("Failed to create window");
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // 60fps max

    let mut buffer = vec![0; width * height];
    for y in 0..height {
        for x in 0..width {
            if ((x / 40) + (y / 40)) % 2 == 1 {
                buffer[y * width + x] = 255 << 16 | 255 << 8 | 255;
            }
        }
    }

    let mut prev_update_time = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        if (now - prev_update_time).as_millis() >= 16 {
            let mut pipeline = &mut redis::pipe();
            let pixels_in_queue = redis_connection.llen::<&str, usize>("turbocow_pixels")
                .expect("Failed to get total pixels in queue");
            for _ in 0..pixels_in_queue {
                pipeline = pipeline.lpop("turbocow_pixels".to_string());
            }

            let result: Vec<Vec<u8>> = pipeline.query(&mut redis_connection).unwrap();
            for pixel_message in result {
                if pixel_message.len() > 0 {
                    let pixel_message: DistributedMessage = bincode::deserialize(&pixel_message)
                        .expect("Failed to deserialize message as distributed message");

                    match pixel_message {
                        DistributedMessage::SetPixel { x, y, r, g, b } => {
                            buffer[y * width + x] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                        },
                        other => panic!("Unexpected message in pixel queue: {:?}"),
                    }
                }
            }

            prev_update_time = now;
        }

        window.update_with_buffer(&buffer, width, height)
            .expect("failed to update window with buffer");
    }
}

fn run_status() {
    info!("connecting to redis...");
    let (_, mut redis_connection) = connect_to_redis();
    info!("connected to redis");

    let result: Vec<u8> = redis_connection.get("turbocow_scene").expect("Failed to get scene from redis");

    if result.len() == 0 {
        info!("Status: no scene set");
        return;
    } else {
        info!("Status: scene set ({} bytes)", result.len());
    }

    let scene = sceneformat::decode(&result).expect("Failed to decode scene");
    let (width, height) = match scene.render_options {
        Some(v) => (v.width, v.height),
        None => (1000, 1000)
    };

    let active_tasks: usize = redis_connection.llen("turbocow_tasks").expect("Failed to get total tasks from redis");
    let total_tasks = width * height;
    let completeness = ((active_tasks as f64 * 100.0) / (total_tasks as f64)) as u16;

    info!("total tasks: {} ({}%)", active_tasks, completeness);

    let pixels_in_queue: usize = redis_connection.llen("turbocow_pixels").expect("Failed to get total pixels from redis");
    info!("total pixels in queue: {}", pixels_in_queue);
}

fn run_reset() {
    let (_, mut redis_connection) = connect_to_redis();
    redis_connection.del::<String, ()>("turbocow_scene".to_string()).expect("Failed to delete task from redis");
    redis_connection.del::<String, ()>("turbocow_tasks".to_string()).expect("Failed to delete tasks from redis");
    redis_connection.del::<String, ()>("turbocow_pixels".to_string()).expect("Failed to delete pixels from redis");
    info!("Completed reset for task");
}

fn connect_to_redis() -> (redis::Client, redis::Connection) {
    let client = redis::Client::open(redis_address()).expect("Failed to connect to redis");
    let redis_connection = client.get_connection().expect("Failed to get redis connection");
    (client, redis_connection)
}

fn redis_address() -> String {
    format!("redis://{}/", var("REDIS_ADDRESS").unwrap_or("127.0.0.1".to_string()))
}