use std::time::Instant;
use std::net::{UdpSocket, TcpListener};
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use minifb::{Window, WindowOptions, Key};
use crossbeam::channel::*;
use std::io::{Read, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::protocol::message::Message;
use crate::protocol::socket::CowSocket;

pub struct WindowOutput {

    window: Window,
    buffer: Vec<u32>,
    cow_socket_server: CowSocket,
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

        WindowOutput {
            window,
            buffer,
            cow_socket_server: CowSocket::start_server(),
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
                if let Some((message, _)) = self.cow_socket_server.recv() {
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
            Message::Batch(messages) =>
                messages.iter().for_each(|message| self.process_message(message)),
            Message::BatchLarge(messages) =>
                messages.iter().for_each(|message| self.process_message(message)),
            Message::SetPixel { x, y, pixel } => {
                let offset = (*y as usize * 1000) + *x as usize;

                self.buffer[offset] = ((pixel.red as u32) << 16)
                    | ((pixel.green as u32) << 8)
                    | (pixel.blue as u32);
            },
            Message::Ping | Message::Pong | Message::Flush | Message::Close => {
                // ignore
            },
        }
    }
}

pub fn run_with_args(args: &[String]) {
    info!("running ui");
    WindowOutput::new().update_loop();
}
