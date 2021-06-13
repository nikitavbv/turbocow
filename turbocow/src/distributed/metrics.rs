use std::env::var;
use std::collections::HashMap;

use lazy_static::lazy_static;
use prometheus::{Registry, IntCounter, TextEncoder, Encoder};
use std::thread::JoinHandle;
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new_custom(
        Some("turbocow_".to_string()),
        {
            let mut labels = HashMap::new();
            labels.insert(
                "hostname".to_string(),
                hostname::get()
                    .expect("Failed to get hostname for metrics").into_string()
                    .expect("Failed to convert hostname to string")
            );
            Some(labels)
        }
    ).expect("Failed to create custom metric registry");
}

pub fn int_counter(name: &str, help: &str) -> IntCounter {
    let counter = IntCounter::new(name, help)
        .expect("Failed to create int counter");
    REGISTRY.register(box counter.clone()).expect("Failed to register counter");
    counter
}

pub fn push_metrics(metrics: String) {
    let client = reqwest::blocking::Client::new();

    let res = client.post(
        metrics_endpoint().expect("Expected METRICS_ENDPOINT to be set")
    )
        .basic_auth(metrics_username(), Some(metrics_password()))
        .body(metrics)
        .send();

    let res = match res {
        Ok(v) => v,
        Err(err) => {
            error!("Failed to send metrics: {:?}", err);
            return;
        }
    };

    if res.status() != 200 && res.status() != 204 {
        warn!("Got error when pushing metrics (status = {})", res.status());
    }
}

pub fn run_metrics_pusher_thread() -> JoinHandle<()> {
    thread::spawn(|| {
        let encoder = TextEncoder::new();

        loop {
            thread::sleep(Duration::from_secs(10));

            let mut buffer = Vec::new();
            encoder.encode(&REGISTRY.gather(), &mut buffer).expect("Failed to encode metrics");

            push_metrics(String::from_utf8_lossy(&buffer).to_string());
        }
    })
}

pub fn metrics_endpoint() -> Option<String> {
    var("METRICS_ENDPOINT").ok()
}

fn metrics_username() -> String {
    var("METRICS_USERNAME").expect("Expected METRICS_USERNAME to be set")
}

fn metrics_password() -> String {
    var("METRICS_PASSWORD").expect("Expected METRICS_PASSWORD to be set")
}