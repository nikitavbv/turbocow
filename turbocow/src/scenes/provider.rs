use crate::scene::scene::Scene;
use std::collections::HashMap;

pub trait SceneProvider {

    fn scene(&self, options: &HashMap<String, String>) -> Scene;
}