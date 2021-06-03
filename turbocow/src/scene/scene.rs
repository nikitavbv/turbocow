use super::{camera::Camera, scene_object::SceneObject};
use crate::scene::light::Light;
use crate::geometry::transform::Transform;
use crate::objects::sphere::Sphere;
use crate::geometry::vector3::Vector3;
use crate::objects::polygon_object::PolygonObject;
use crate::io::traits::ModelLoader;
use crate::Resolve;
use livestonk::Livestonk;
use crate::objects::plane::Plane;
use crate::materials::material::Material;
use turbocow_core::models::pixel::Pixel;

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

    pub fn from_sceneformat(scene: sceneformat::Scene) -> Self {
        let mut s = Scene::new();

        for camera in &scene.cameras {
            let transform = camera.transform.as_ref().map(|v| Transform::from_scene_format(&v)).unwrap_or(Transform::default());
            s.set_camera(
                Camera::default().with_transform(transform)
            );
        }

        for object in &scene.scene_objects {
            s.objects.push(scene_object_from_sceneformat(object));
        }

        s
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

fn scene_object_from_sceneformat(object: &sceneformat::SceneObject) -> Box<dyn SceneObject + Sync + Send> {
    let model_loader: Box<ModelLoader> = Livestonk::resolve();

    let mesh = match &object.mesh {
        Some(v) => v,
        None => panic!("This scene object has no mesh: {}", object.id),
    };

    let transform = match &object.transform {
        Some(v) => Transform::from_scene_format(v),
        None => Transform::new(Vector3::zero(), Vector3::zero()),
    };

    let material = object.object_material.as_ref().map(|v| match v {
        sceneformat::scene_object::ObjectMaterial::MaterialId(_) => panic!("Referencing materials by id is not implemented"),
        sceneformat::scene_object::ObjectMaterial::Material(m) => match m.material.as_ref().unwrap() {
            sceneformat::material::Material::LambertReflection(lambert) => Material::Lambertian { albedo: 0.18, color: Pixel::from_rgb((lambert.color.as_ref().unwrap().r * 255.0).round() as u8, (lambert.color.as_ref().unwrap().g * 255.0).round() as u8, (lambert.color.as_ref().unwrap().b * 255.0).round() as u8) },
            sceneformat::material::Material::SpecularReflection(_) => panic!("Not implemented"),
        }
    }).unwrap_or(Material::Lambertian { albedo: 0.18, color: Pixel::from_rgb(194, 24, 91) });

    match mesh {
        sceneformat::scene_object::Mesh::Sphere(sphere) => {
            box Sphere::new(object.id as usize, transform, material, sphere.radius)
        },
        sceneformat::scene_object::Mesh::MeshedObject(meshed_object) => {
            let model = model_loader.load(&meshed_object.reference).expect("Failed to load model");
            box PolygonObject::from_model(object.id as usize, transform, &model)
        },
        sceneformat::scene_object::Mesh::Plane(_) => {
            box Plane::new(object.id as usize, transform, material)
        }
        other => panic!("This mesh is not implemented: {:?}", other),
    }
}