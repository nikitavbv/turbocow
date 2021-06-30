use std::net::{UdpSocket, SocketAddrV4, Ipv4Addr, TcpStream, TcpListener, SocketAddr};
use std::thread::JoinHandle;
use std::thread;

use crossbeam::channel::*;
use custom_error::custom_error;

use crate::protocol::message::Message;
use std::time::{Instant, Duration};
use std::convert::TryInto;
use std::io::{Write, Read, Cursor};
use byteorder::{WriteBytesExt, LittleEndian, ReadBytesExt};
use std::sync::{Arc, Mutex};

custom_error! {pub CowSocketError
    FailedToConnect {description: String} = "Failed to connect: {description}",
}

pub struct CowSocket {

    udp_thread: JoinHandle<()>,
    tcp_thread: JoinHandle<()>,

    message_sender_udp: Option<Sender<(Message, MessageMetadata)>>,
    message_sender_tcp: Option<Sender<(Message, MessageMetadata)>>,
    message_receiver: Option<Receiver<(Message, MessageMetadata)>>,
}

#[derive(Clone, Debug)]
pub struct MessageMetadata {

    pub guaranteed_delivery: bool,
    pub target: Option<SocketAddr>,
}

impl CowSocket {

    pub fn start_server() -> Self {
        let (tcp_send_tx, tcp_send_rx) = crossbeam::channel::unbounded();
        let (udp_send_tx, udp_send_rx) = crossbeam::channel::unbounded();
        let (receive_tx, receive_rx) = crossbeam::channel::unbounded();

        CowSocket {
            udp_thread: start_udp_server(udp_send_rx, udp_send_tx.clone(), receive_tx.clone()),
            tcp_thread: start_tcp_server(tcp_send_rx, tcp_send_tx.clone(), receive_tx),
            message_sender_udp: Some(udp_send_tx),
            message_sender_tcp: Some(tcp_send_tx),
            message_receiver: Some(receive_rx),
        }
    }

    pub fn start_client(target: Ipv4Addr) -> Result<Self, CowSocketError> {
        let (udp_send_tx, udp_send_rx) = crossbeam::channel::unbounded();
        let (tcp_send_tx, tcp_send_rx) = crossbeam::channel::unbounded();
        let (receive_tx, receive_rx) = crossbeam::channel::unbounded();

        let connected_to = Some(SocketAddrV4::new(target, 30421).into());

        Ok(CowSocket {
            udp_thread: start_udp_client(target, udp_send_rx, receive_tx.clone(), connected_to),
            tcp_thread: start_tcp_client(target, tcp_send_rx, receive_tx)?,
            message_sender_udp: Some(udp_send_tx),
            message_sender_tcp: Some(tcp_send_tx),
            message_receiver: Some(receive_rx),
        })
    }

    pub fn recv(&self) -> Option<(Message, MessageMetadata)> {
        if let Some(msg) = self.message_receiver.as_ref().unwrap().try_recv().ok() {
            Some(msg)
        } else {
            None
        }
    }

    pub fn recv_blocking(&self) -> Option<(Message, MessageMetadata)> {
        if let Some(msg) = self.message_receiver.as_ref().unwrap().recv().ok() {
            Some(msg)
        } else {
            None
        }
    }

    pub fn send_with_metadata(&self, message: Message, metadata: MessageMetadata) {
        if metadata.guaranteed_delivery {
            &self.message_sender_tcp
        } else {
            &self.message_sender_udp
        }.as_ref()
            .expect("Send channel is not set for this message type")
            .send((message, metadata)).expect("Failed to write message to send channel");
    }

    pub fn send(&self, message: Message, guaranteed_delivery: bool) {
        &self.send_with_metadata(message, MessageMetadata {
            guaranteed_delivery,
            target: None,
        });
    }

    pub fn flush(&self) {
        &self.send(Message::Flush, false);
        &self.send(Message::Flush, true);
    }

    pub fn close(self) {
        &self.send(Message::Close, false);
        &self.send(Message::Close, true);
        self.udp_thread.join().unwrap();
        self.tcp_thread.join().unwrap();
    }
}

fn start_udp_server(rx: Receiver<(Message, MessageMetadata)>, send_tx: Sender<(Message, MessageMetadata)>, tx: Sender<(Message, MessageMetadata)>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket_read = UdpSocket::bind("0.0.0.0:30421").unwrap();
        let mut socket_write = socket_read.try_clone().unwrap();
        info!("cow server udp socket is started");

        let rx_handle = thread::spawn(move || udp_sender_handler(socket_write, rx, None));
        udp_receiver_handler(socket_read, tx.clone());

        send_tx.send((Message::Close, MessageMetadata {
            guaranteed_delivery: false,
            target: None,
        })).unwrap();

        rx_handle.join().unwrap();
    })
}

fn start_tcp_server(mut rx: Receiver<(Message, MessageMetadata)>, send_tx: Sender<(Message, MessageMetadata)>, tx: Sender<(Message, MessageMetadata)>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut socket = TcpListener::bind("0.0.0.0:30422").unwrap();
        info!("cow server tcp socket is started");

        for stream in socket.incoming() {
            info!("new tcp connection established");
            let mut stream_read = stream.unwrap();
            let mut stream_write = stream_read.try_clone().unwrap();

            let rx_handle = thread::spawn(move || tcp_sender_handler(stream_write, rx));
            tcp_receiver_handler(stream_read, tx.clone());

            send_tx.send((Message::Close, MessageMetadata {
                guaranteed_delivery: false,
                target: None,
            })).unwrap();

            rx = rx_handle.join().unwrap();
        }
    })
}

fn start_udp_client(target: Ipv4Addr, rx: Receiver<(Message, MessageMetadata)>, tx: Sender<(Message, MessageMetadata)>, default_target: Option<SocketAddr>) -> JoinHandle<()> {
    thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
        let receiver_socket = socket.try_clone().unwrap();

        let receiver_handle = thread::spawn(move || udp_receiver_handler(receiver_socket, tx));

        udp_sender_handler(socket, rx, default_target);

        receiver_handle.join().unwrap();
    })
}

fn start_tcp_client(target: Ipv4Addr, rx: Receiver<(Message, MessageMetadata)>, tx: Sender<(Message, MessageMetadata)>) -> Result<JoinHandle<()>, CowSocketError> {
    let target = SocketAddrV4::new(target, 30422);
    let socket = TcpStream::connect(target).map_err(|err| CowSocketError::FailedToConnect {
        description: format!("Failed to connect to server: {:?}", err)
    })?;
    socket.set_nodelay(true).unwrap();

    let receiver_stream = socket.try_clone().unwrap();
    let receiver_handle = thread::spawn(move || {
        tcp_receiver_handler(receiver_stream, tx)
    });

    Ok(thread::spawn(move || {
        tcp_sender_handler(socket, rx);
        receiver_handle.join().unwrap()
    }))
}

fn udp_receiver_handler(socket: UdpSocket, tx: Sender<(Message, MessageMetadata)>) {
    let mut buffer = [0; 65536];

    loop {
        let (total_read, from) = socket.recv_from(&mut buffer).unwrap();
        if total_read > 0 {
            tx.send((bincode::deserialize(&buffer[0..total_read]).unwrap(), MessageMetadata {
                guaranteed_delivery: false,
                target: Some(from),
            })).unwrap();
        }
    }
}

fn udp_sender_handler(socket: UdpSocket, rx: Receiver<(Message, MessageMetadata)>, default_target: Option<SocketAddr>) {
    let mut messages_to_send: Vec<(Message, MessageMetadata)> = Vec::with_capacity(32);
    let mut total_messages = 0;
    let mut last_push = Instant::now();
    let mut flush = false;
    let mut close = false;

    loop {
        if let Ok((message, metadata)) = rx.recv() {
            match message {
                Message::Flush => {
                    flush = true;
                },
                Message::Close => {
                    flush = true;
                    close = true;
                },
                other => messages_to_send.push((other, metadata))
            };
        } else if messages_to_send.len() == 0 {
            break;
        } else {
            flush = true;
        }
        let time = Instant::now();
        total_messages += 1;

        if ((time - last_push).as_millis() > 20 || total_messages >= 4096 || flush) && messages_to_send.len() > 0 {
            for (message, metadata) in messages_to_send.iter() {
                let serialized = &bincode::serialize(&message).unwrap();

                if serialized.len() > 65000 {
                    panic!("serialized size is too big for udp datagram: {}", serialized.len());
                }

                socket.send_to(serialized, metadata.target.unwrap_or_else(|| default_target.unwrap())).unwrap();
            }

            messages_to_send.clear();
            total_messages = 0;
            last_push = time;
        }

        if close {
            break;
        }

        if messages_to_send.len() == 32 {
            let metadata = messages_to_send[0].1.clone();
            let multi_message = Message::Batch(box messages_to_send.iter().map(|v| v.0.clone()).collect::<Vec<Message>>().try_into().unwrap());
            messages_to_send = Vec::with_capacity(32);
            messages_to_send.push((multi_message, metadata));
            total_messages += 1;
        }
    }
}

fn tcp_receiver_handler(mut stream: TcpStream, tx: Sender<(Message, MessageMetadata)>) {
    let mut buffer = [0; 65536];
    let mut all_data = Vec::new();

    loop {
        let total_read = stream.read(&mut buffer).unwrap();
        if total_read > 0 {
            all_data.append(&mut buffer[0..total_read].to_vec());

            if all_data.len() > 0 {
                let mut len_bytes = Cursor::new(all_data[0..4].to_vec());
                let part_len = len_bytes.read_u32::<LittleEndian>().unwrap() as usize;
                if all_data.len() >= part_len + 4 {
                    all_data.drain(0..4);

                    let message = bincode::deserialize(&all_data.drain(0..part_len).collect::<Vec<u8>>()).unwrap();
                    tx.send((message, MessageMetadata {
                        guaranteed_delivery: true,
                        target: None,
                    })).unwrap();
                }
            }
        } else {
            info!("tcp connection closed");
            break;
        }
    }
}

fn tcp_sender_handler(mut socket: TcpStream, rx: Receiver<(Message, MessageMetadata)>) -> Receiver<(Message, MessageMetadata)> {
    let mut messages_to_send: Vec<Message> = Vec::with_capacity(32);
    let mut total_messages = 0;
    let mut last_push = Instant::now();
    let mut flush = false;
    let mut close = false;

    loop {
        if let Ok((message, metadata)) = rx.recv() {
            match message {
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

        if ((time - last_push).as_millis() > 20 || total_messages >= 100000 || flush) && messages_to_send.len() > 0 {
            let message = if messages_to_send.len() == 1 {
                messages_to_send.remove(0)
            } else {
                Message::BatchLarge(messages_to_send.clone())
            };
            let serialized = &bincode::serialize(&message).unwrap();
            let mut len_bytes = Vec::with_capacity(4);
            len_bytes.write_u32::<LittleEndian>(serialized.len() as u32).unwrap();
            socket.write(&len_bytes).unwrap();
            socket.write(serialized).unwrap();
            messages_to_send.clear();
            total_messages = 0;
            last_push = time;
        }

        if close {
            break;
        }
    }

    rx
}
