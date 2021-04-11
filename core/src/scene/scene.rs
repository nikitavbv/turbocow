use super::{camera::Camera, scene_object::SceneObject};

pub struct Scene {
    camera: Option<Camera>,
    objects: Vec<Box<dyn SceneObject>>,
}

impl Scene {

    pub fn new() -> Self {
        Self {
            camera: None,
            objects: Vec::new(),
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
    }

    pub fn add_object(&mut self, obj: Box<dyn SceneObject>) {
        self.objects.push(obj)
    }

    pub fn objects(&self) -> &Vec<Box<dyn SceneObject>> {
        &self.objects
    }

    pub fn camera(&self) -> &Camera {
        self.camera.as_ref().expect("expected camera to be present")
    }
}