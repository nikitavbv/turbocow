#![feature(box_syntax)]

mod example;

use example::{APIController, Database, Postgres, Mongo, WebController};
use livestonk::{Livestonk, Resolve};

livestonk::bind_to_instance!(dyn Database, Postgres {});
livestonk::bind!(dyn WebController, APIController);

fn main() {
    let controller: Box<dyn WebController> = Livestonk::resolve();
    controller.process_request();
}
