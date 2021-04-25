use std::net::{UdpSocket, SocketAddrV4, Ipv4Addr, TcpStream, TcpListener};
use std::thread::JoinHandle;
use std::thread;

use crossbeam::channel::*;
use custom_error::custom_error;

use crate::protocol::message::Message;
use std::time::{Instant, Duration};
use std::convert::TryInto;
use std::io::{Write, Read, Cursor};
use byteorder::{WriteBytesExt, LittleEndian, ReadBytesExt};

custom_error! {pub CowSocketError
    FailedToConnect {description: String} = "Failed to connect: {description}",
}

pub struct CowSocket {

    udp_thread: JoinHandle<()>,
    tcp_thread: JoinHandle<()>,

    message_sender_udp: Option<Sender<Message>>,
    message_sender_tcp: Option<Sender<Message>>,
    message_receiver_udp: Option<Receiver<Message>>,
    message_receiver_tcp: Option<Receiver<Message>>,
}

impl CowSocket {

    pub fn start_server() -> Self {
        let (udp_tx, udp_rx) = crossbeam::channel::unbounded();
        let (tcp_tx, tcp_rx) = crossbeam::channel::unbounded();

        CowSocket {
            udp_thread: start_udp_server(udp_tx),
            tcp_thread: start_tcp_server(tcp_tx),
            message_sender_udp: None,
            message_sender_tcp: None,
            message_receiver_udp: Some(udp_rx),
            message_receiver_tcp: Some(tcp_rx),
        }
    }

    pub fn start_client(target: Ipv4Addr) -> Result<Self, CowSocketError> {
        let (udp_tx, udp_rx) = crossbeam::channel::unbounded();
        let (tcp_tx, tcp_rx) = crossbeam::channel::unbounded();

        Ok(CowSocket {
            udp_thread: start_udp_client(target, udp_rx),
            tcp_thread: start_tcp_client(target, tcp_rx)?,
            message_sender_udp: Some(udp_tx),
            message_sender_tcp: Some(tcp_tx),
            message_receiver_udp: None,
            message_receiver_tcp: None,
        })
    }

    pub fn recv(&self) -> Option<Message> {
        if let Some(msg) = self.message_receiver_udp.as_ref().unwrap().try_recv().ok() {
            Some(msg)
        } else if let Some(msg) = self.message_receiver_tcp.as_ref().unwrap().try_recv().ok() {
            Some(msg)
        } else {
            None
        }
    }

    pub fn send(&self, message: Message, guaranteed_delivery: bool) {
        if guaranteed_delivery {
            &self.message_sender_tcp
        } else {
            &self.message_sender_udp
        }.as_ref().unwrap().send(message).unwrap()
    }

    pub fn flush(&self) {
        &self.send(Message::Flush, false);
        &self.send(Message::Flush, true);
    }

    pub fn close(self) {
        &self.send(Message::Close, false);
        &self.send(Message::Close, true);
        self.udp_thread.join();
        self.tcp_thread.join();
    }
}

fn start_udp_server(tx: Sender<Message>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = UdpSocket::bind("0.0.0.0:30421").unwrap();
        info!("cow server udp socket is started");

        let mut buffer = [0; 65536];

        loop {
            let total_read = socket.recv(&mut buffer).unwrap();
            if total_read > 0 {
                tx.send(bincode::deserialize(&buffer[0..total_read]).unwrap()).unwrap();
            }
        }
    })
}

fn start_tcp_server(tx: Sender<Message>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = TcpListener::bind("0.0.0.0:30422").unwrap();
        info!("cow server tcp socket is started");

        let mut buffer = [0; 65536];
        let mut all_data = Vec::new();

        for stream in socket.incoming() {
            info!("new tcp connection established");
            let mut stream = stream.unwrap();

            loop {
                let total_read = stream.read(&mut buffer).unwrap();
                if total_read > 0 {
                    all_data.append(&mut buffer[0..total_read].to_vec());

                    if all_data.len() > 0 {
                        let mut len_bytes = Cursor::new(all_data[0..4].to_vec());
                        let part_len = len_bytes.read_u32::<LittleEndian>().unwrap() as usize;
                        if all_data.len() >= part_len + 4 {
                            all_data.drain(0..4);
                            tx.send(bincode::deserialize(&all_data.drain(0..part_len).collect::<Vec<u8>>()).unwrap()).unwrap();
                        }
                    }
                } else {
                    info!("tcp connection closed");
                    break;
                }
            }
        }
    })
}

fn start_udp_client(target: Ipv4Addr, rx: Receiver<Message>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        socket.set_nonblocking(true);

        let target = SocketAddrV4::new(target, 30421);

        let mut messages_to_send: Vec<Message> = Vec::with_capacity(32);
        let mut total_messages = 0;
        let mut last_push = Instant::now();
        let mut flush = false;
        let mut close = false;

        loop {
            if let Ok(v) = rx.recv() {
                match v {
                    Message::Flush => {
                        flush = true;
                    },
                    Message::Close => {
                        flush = true;
                        close = true;
                    },
                    other => messages_to_send.push(other)
                };
            } else if messages_to_send.len() == 0 {
                break;
            } else {
                flush = true;
            }
            let time = Instant::now();
            total_messages += 1;

            if (time - last_push).as_millis() > 20 || total_messages >= 4096 || flush {
                for message in messages_to_send.iter() {
                    let serialized = &bincode::serialize(&message).unwrap();

                    if serialized.len() > 65000 {
                        panic!("serialized size is too big for udp datagram: {}", serialized.len());
                    }

                    socket.send_to(serialized, target);
                }

                messages_to_send.clear();
                total_messages = 0;
                last_push = time;
            }

            if close {
                break;
            }

            if messages_to_send.len() == 32 {
                let multi_message = Message::Batch(box messages_to_send.try_into().unwrap());
                messages_to_send = Vec::with_capacity(32);
                messages_to_send.push(multi_message);
                total_messages += 1;
            }
        }
    })
}

fn start_tcp_client(target: Ipv4Addr, rx: Receiver<Message>) -> Result<JoinHandle<()>, CowSocketError> {
    let target = SocketAddrV4::new(target, 30422);
    let mut socket = TcpStream::connect(target).map_err(|err| CowSocketError::FailedToConnect {
        description: format!("Failed to connect to server: {:?}", err)
    })?;
    socket.set_nonblocking(true);
    socket.set_nodelay(true);

    Ok(thread::spawn(move || {
        let mut messages_to_send: Vec<Message> = Vec::with_capacity(32);
        let mut total_messages = 0;
        let mut last_push = Instant::now();
        let mut flush = false;
        let mut close = false;

        loop {
            if let Ok(v) = rx.recv() {
                match v {
                    Message::Flush => {
                        flush = true;
                    },
                    Message::Close => {
                        flush = true;
                        close = true;
                    },
                    other => messages_to_send.push(other)
                };
            } else if messages_to_send.len() == 0 {
                break;
            } else {
                flush = true;
            }
            let time = Instant::now();
            total_messages += 1;

            if (time - last_push).as_millis() > 20 || total_messages >= 100000 || flush {
                let message = if messages_to_send.len() == 1 {
                    messages_to_send.remove(0)
                } else {
                    Message::BatchLarge(messages_to_send.clone())
                };
                let serialized = &bincode::serialize(&message).unwrap();
                let mut len_bytes = Vec::with_capacity(4);
                len_bytes.write_u32::<LittleEndian>(serialized.len() as u32);
                socket.write(&len_bytes);
                socket.write(serialized);
                messages_to_send.clear();
                total_messages = 0;
                last_push = time;
            }

            if close {
                break;
            }
        }
    }))
}