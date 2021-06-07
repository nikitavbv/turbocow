use std::collections::HashMap;
use crate::scenes::provider::SceneProvider;
use livestonk::Livestonk;
use crate::scenes::sceneformat::SceneFormatLoader;
use sceneformat::{read, save};

pub fn run_pack(options: HashMap<String, String>) {
    let scene = read(&options.get("source").expect("expected source to be set"))
        .expect("failed to read scene");
    save(&scene, &options.get("target").expect("expected target to be set"))
        .expect("failed to save scene");
    info!("done packing scene");
}