use crate::scene::scene::Scene;

pub trait SceneProvider {

    fn scene(&self) -> Scene;
}