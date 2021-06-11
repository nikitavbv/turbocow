use std::collections::HashMap;
use std::env::var;

use redis::{Commands, RedisError, Connection};
use serde::{Serialize, Deserialize};
use indicatif::{ProgressBar, ProgressIterator};
use crate::render::basic::render_pixel;
use crate::scene::scene::Scene;

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