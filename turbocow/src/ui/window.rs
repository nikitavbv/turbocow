use std::time::Instant;
use std::net::UdpSocket;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use minifb::{Window, WindowOptions, Key};
use crossbeam::channel::*;
use crate::protocol::Message;

pub struct WindowOutput {

    window: Window,
    buffer: Vec<u32>,
    rx: Receiver<Message>,
    udp_server_handle: JoinHandle<()>,
}

impl WindowOutput {

    pub fn new() -> Self {
        let mut window = Window::new(
            "turbocow",
            1000,
            1000,
            WindowOptions::default()
        ).unwrap();
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600))); // 60fps max

        let mut buffer = vec![0; 1000 * 1000];
        for y in 0..1000 {
            for x in 0..1000 {
                if ((x / 40) + (y / 40)) % 2 == 1 {
                    buffer[y * 1000 + x] = 255 << 16 | 255 << 8 | 255;
                }
            }
        }

        let (tx, rx) = crossbeam::channel::unbounded();

        WindowOutput {
            window,
            buffer,
            rx,
            udp_server_handle: start_udp_server(tx),
        }
    }

    pub fn update_loop(&mut self) {
        let mut prev_second = 0;
        let mut prev_second_updates = 0;
        let start = Instant::now();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            let diff = Instant::now() - start;
            if diff.as_secs() == prev_second {
                prev_second_updates += 1;
            } else {
                self.window.set_title(format!("turbocow, fps: {}", prev_second_updates).as_str());
                prev_second_updates = 1;
                prev_second = diff.as_secs();
            }

            loop {
                if let Ok(message) = self.rx.try_recv() {
                    self.process_message(&message);
                } else {
                    break;
                }
            }
            self.window.update_with_buffer(&self.buffer, 1000, 1000).unwrap();
        }
    }

    pub fn process_message(&mut self, message: &Message) {
        match message {
            Message::Multi(messages) => {
                messages.iter().for_each(|message| self.process_message(message));
            },
            Message::SetPixel { x, y, pixel } => {
                let offset = (*y as usize * 1000) + *x as usize;

                self.buffer[offset] = ((pixel.red as u32) << 16)
                | ((pixel.green as u32) << 8)
                | (pixel.blue as u32);

                //buffer[offset] = pixel.red as u32;
                //buffer[offset + 1] = pixel.green as u32;
                //buffer[offset + 2] = pixel.blue as u32;
            }
        }
    }
}

pub fn run_with_args(args: &[String]) {
    info!("running ui");
    WindowOutput::new().update_loop();
}

fn start_udp_server(tx: Sender<Message>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = UdpSocket::bind("0.0.0.0:30421").unwrap();
        let mut udp_incoming = [0; 4096];

        info!("udp server started");

        loop {
            let (bytes_read, from) = socket.recv_from(&mut udp_incoming).unwrap();
            tx.send(bincode::deserialize(&udp_incoming[0..bytes_read]).unwrap()).unwrap();
        }
    })
}