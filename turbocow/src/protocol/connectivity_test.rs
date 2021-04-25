use std::thread;

use crate::protocol::socket::CowSocket;
use std::time::Duration;
use std::net::Ipv4Addr;
use crate::protocol::message::Message;

pub fn run_with_args(args: &[String]) {
    if args.len() == 0 {
        error!("Please specify mode: \"server\" or \"client\"");
        return;
    }

    match args[0].as_str() {
        "server" => run_server(),
        "client" => run_client(&args[1..]),
        _ => error!("Unknown mode, valid ones are \"server\" or \"client\"")
    }
}

fn run_server() {
    let server = CowSocket::start_server();
    loop {
        if let Some(message) = server.recv() {
            info!("Received message: {:?}", message);
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn run_client(args: &[String]) {
    let client = CowSocket::start_client(Ipv4Addr::new(127, 0, 0, 1)).unwrap();
    client.send(Message::Ping, true);
    client.send(Message::Ping, false);
    client.close();
}