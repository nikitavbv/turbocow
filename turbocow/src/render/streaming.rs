use livestonk::*;
use crate::Resolve;

use crate::protocol::socket::{CowSocket, MessageMetadata};
use crate::protocol::message::Message;
use crate::scenes::provider::SceneProvider;
use crate::render::render::Render;
use crate::render::multithreaded::MultithreadedRender;
use turbocow_core::models::image::Image;
use bincode::options;
use std::collections::HashMap;
use std::thread;

pub fn run_streaming_render() {
    let socket = CowSocket::start_server();

    if let Some((message, metadata)) = socket.recv_blocking() {
        match message {
            Message::StartStreaming => start_streaming(socket, metadata),
            other => error!("Unexpected streaming server message: {:?}", other),
        }
    }
}

fn start_streaming(socket: CowSocket, metadata: MessageMetadata) {
    info!("starting streaming session...");

    let scene_provider: Box<dyn SceneProvider> = Livestonk::resolve();
    let render: Box<MultithreadedRender> = Livestonk::resolve();
    let scene = scene_provider.scene(&HashMap::new());

    let (image_tx, image_rx) = crossbeam::channel::unbounded::<Image>();
    let stream_handle = thread::spawn(move || {
        loop {
            if let Ok(image) = image_rx.recv() {
                for y in 0..image.height as u16 {
                    for x in 0..image.width as u16 {
                        socket.send(Message::SetPixel {
                            x,
                            y,
                            pixel: image.get_pixel(x as usize, y as usize)
                        }, true);
                    }
                }
            }
        }
    });

    let mut output = Image::new(1000, 1000);

    loop {
        info!("rendering image");

        if let Err(err) = render.render(&scene, &mut output) {
            warn!("failed to render: {:?}", err);
            continue;
        }

        image_tx.send(output.clone());
    }

    stream_handle.join();
}