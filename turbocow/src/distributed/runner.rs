use std::collections::HashMap;
use std::env::var;

use redis::{Commands, RedisError, Connection, ConnectionLike};
use serde::{Serialize, Deserialize};
use indicatif::{ProgressBar, ProgressIterator};
use crate::render::basic::render_pixel;
use crate::scene::scene::Scene;
use minifb::{Window, WindowOptions, Key};
use std::time::Instant;

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

fn run_init(options: &HashMap<String, String>) {
    info!("Loading scene");
    let scene = sceneformat::read(
        options.get("source").expect("Expected source .cowscene to be set")
    ).expect("Failed to read cowscene file");
    let scene_binary = sceneformat::encode(&scene)
        .expect("Failed to encode .cowscene to binary");

    info!("Creating a new distributed task");
    let (_, mut redis_connection) = connect_to_redis();
    redis_connection.set::<String, Vec<u8>, ()>("turbocow_scene".to_string(), scene_binary)
        .expect("Failed to save turbocow_scene to redis");

    let (width, height) = match scene.render_options {
        Some(v) => (v.width as usize, v.height as usize),
        None => (1000, 1000)
    };

    for y in (0..height).progress() {
        let mut pipeline = &mut redis::pipe();

        for x in 0..width {
            let message = bincode::serialize(&DistributedMessage::ProcessPixel(x, y))
                .expect("failed to serialize distributed message");
            pipeline = pipeline.rpush::<String, Vec<u8>>("turbocow_tasks".to_string(), message)
                .ignore();
        }

        pipeline.query::<()>(&mut redis_connection).expect("failed to send a pipeline query to redis");
    }
}

fn run_worker() {
    info!("connecting to redis...");
    let (_, mut redis_connection) = connect_to_redis();
    info!("connected to redis");

    let result: Vec<u8> = redis_connection.get("turbocow_scene").expect("Failed to get scene from redis");
    let scene = sceneformat::decode(&result).expect("Failed to load sceneformat scene");
    let (viewport_width, viewport_height) = match &scene.render_options {
        Some(v) => (v.width as usize, v.height as usize),
        None => (1000, 1000),
    };
    let scene = Scene::from_sceneformat(scene);

    loop {
        let task: Vec<u8> = redis_connection.lpop("turbocow_tasks").expect("Failed to get task from redis");
        let task: DistributedMessage = bincode::deserialize(&task).expect("Failed to deserialize task");

        match task {
            DistributedMessage::ProcessPixel(x, y) => worker_process_pixel(&mut redis_connection, &scene, viewport_width, viewport_height, x, y),
            other => panic!("Did not expect this message in tasks queue: {:?}", other),
        }
    }
}

fn worker_process_pixel(redis_connection: &mut redis::Connection, scene: &Scene, viewport_width: usize, viewport_height: usize, x: usize, y: usize) {
    info!("processing pixel: {} {}", x, y);
    let pixel = render_pixel(scene, viewport_width, viewport_height, x, y);
    info!("Pixel is: {:?}", pixel);

    let message = bincode::serialize(&DistributedMessage::SetPixel {
        x,
        y,
        r: pixel.red,
        g: pixel.green,
        b: pixel.blue,
    }).expect("Failed to serialize pixel message");
    redis_connection.rpush::<String, Vec<u8>, ()>("turbocow_pixels".to_string(), message).expect("Failed to send pixel to redis queue");
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