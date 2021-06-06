use std::collections::HashMap;
use std::fs;
use sceneformat::read;
use livestonk::Component;
use crate::scenes::provider::SceneProvider;
use crate::scene::scene::Scene;

#[derive(Component)]
pub struct SceneFormatLoader {
}

impl SceneProvider for SceneFormatLoader {

    fn scene(&self, options: &HashMap<String, String>) -> Scene {
        let scene_file = options.get("source").expect("--source option is not set");

        if !scene_file.ends_with(".cowscene") {
            panic!("Expected .cowscene file as a source");
        }

        let scene = read(scene_file).expect("failed to read sceneformat file");
        Scene::from_sceneformat(scene)
    }
}