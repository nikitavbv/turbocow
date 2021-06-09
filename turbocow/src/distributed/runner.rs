use std::collections::HashMap;
use std::env::var;

use redis::{Commands, RedisError};

pub fn run_distributed(commands: &[String], options: &HashMap<String, String>) {
    match commands[0].as_str() {
        "init" => run_init(options),
        "status" => run_status(),
        "reset" => run_reset(),
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
    redis_connection.set::<String, Vec<u8>, ()>("turbocow_task".to_string(), scene_binary)
        .expect("Failed to save turbocow_task to redis");
}

fn run_status() {
    let (_, mut redis_connection) = connect_to_redis();
    let result: Vec<u8> = redis_connection.get("turbocow_task").expect("Failed to get task from redis");

    if result.len() == 0 {
        info!("Status: no task set");
    } else {
        info!("Status: task set ({} bytes)", result.len());
    }
}

fn run_reset() {
    let (_, mut redis_connection) = connect_to_redis();
    redis_connection.del::<String, ()>("turbocow_task".to_string()).expect("Failed to delete task from redis");
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