use minifb::{Window, WindowOptions, Key};
use std::time::Instant;
use std::net::UdpSocket;
use std::thread;
use std::thread::JoinHandle;

pub fn run_with_args(args: &[String]) {
    info!("running ui");

    let mut buffer: Vec<u32> = vec![0; 1280 * 720];

    let mut window = Window::new(
        "turbocow",
        1280,
        720,
        WindowOptions::default()
    ).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // 60fps max

    let udp_server_handle = start_udp_server();

    let mut prev_second = 0;
    let mut prev_second_updates = 0;
    let start = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let diff = Instant::now() - start;
        if diff.as_secs() == prev_second {
            prev_second_updates += 1;
        } else {
            window.set_title(format!("turbocow, fps: {}", prev_second_updates).as_str());
            prev_second_updates = 1;
            prev_second = diff.as_secs();
        }

        window.update_with_buffer(&buffer, 1280, 720).unwrap();
    }

    udp_server_handle.join();
}

fn start_udp_server() -> JoinHandle<u32> {
    thread::spawn(|| {
        let mut socket = UdpSocket::bind("0.0.0.0:30421").unwrap();
        let mut udp_incoming = [0; 4096];

        info!("udp server started");

        loop {
            let (bytes_read, from) = socket.recv_from(&mut udp_incoming).unwrap();
            info!("read {} bytes from {}", bytes_read, from);
        }
    })
}