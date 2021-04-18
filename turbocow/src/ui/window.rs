use std::time::Instant;
use std::net::{UdpSocket, TcpListener};
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex, RwLock, mpsc};
use minifb::{Window, WindowOptions, Key};
use crossbeam::channel::*;
use crate::protocol::Message;
use std::io::{Read, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};

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
        let listener = TcpListener::bind("0.0.0.0:30421").unwrap();
        info!("window tcp server started");

        let mut buffer = [0; 2048];
        let mut all_data = Vec::new();

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();

            loop {
                let total_read = stream.read(&mut buffer).unwrap();

                if total_read > 0 {
                    all_data.append(&mut buffer[0..total_read].to_vec());

                    let mut size_bytes = Cursor::new(all_data[0..4].to_vec());
                    let size = size_bytes.read_u32::<LittleEndian>().unwrap();
                    if all_data.len() >= size as usize + 4 {
                        all_data.drain(0..4);
                        let data: Vec<u8> = all_data.drain(0..(size as usize)).collect();
                        tx.send(bincode::deserialize(&data).unwrap()).unwrap();
                    }
                }
            }
        }
    })
}