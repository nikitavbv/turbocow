use super::{camera::Camera, scene_object::SceneObject};
use crate::scene::light::Light;

pub struct Scene {
    camera: Option<Camera>,
    objects: Vec<Box<dyn SceneObject + Sync + Send>>,
    lights: Vec<Box<dyn Light + Sync + Send>>,
}

impl Scene {

    pub fn new() -> Self {
        Self {
            camera: None,
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = Some(camera);
    }

    pub fn camera(&self) -> &Camera {
        self.camera.as_ref().expect("expected camera to be present")
    }

    pub fn add_object(&mut self, obj: Box<dyn SceneObject + Sync + Send>) {
        self.objects.push(obj)
    }

    pub fn objects(&self) -> &Vec<Box<dyn SceneObject + Sync + Send>> {
        &self.objects
    }

    pub fn add_light(&mut self, light: Box<dyn Light + Sync + Send>) {
        self.lights.push(light)
    }

    pub fn lights(&self) -> &Vec<Box<dyn Light + Sync + Send>> {
        &self.lights
    }
}