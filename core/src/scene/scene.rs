use super::{camera::Camera, scene_object::SceneObject};

pub struct Scene {
    camera: Option<Camera>,
    objects: Vec<Box<dyn SceneObject>>,
}

impl Scene {

    pub fn new() -> Self {
        Self {
            camera: None,
        }
    }

    pub fn add_object(obj: Box<dyn SceneObject>) {
        // TODO: finish writing this   
    }

    pub fn camera(&self) -> &Camera {
        self.camera.as_ref().expect("expected camera to be present")
    }
}