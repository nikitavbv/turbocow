use livestonk::{Component, Livestonk};
use crate::scenes::provider::SceneProvider;
use crate::scene::scene::Scene;
use crate::scene::camera::Camera;
use crate::objects::polygon_object::PolygonObject;
use crate::geometry::transform::Transform;
use crate::geometry::vector3::Vector3;
use crate::io::traits::ModelLoader;
use crate::objects::sphere::Sphere;
use crate::scene::distant_light::DistantLight;
use std::collections::HashMap;
use crate::objects::cube::Cube;
use crate::scene::point_light::PointLight;
use crate::objects::plane::Plane;
use crate::materials::material::Material;
use turbocow_core::models::pixel::Pixel;
use crate::Resolve;

#[derive(Component)]
pub struct DemoSceneProvider {
    model_loader: Box<dyn ModelLoader>,
}

impl SceneProvider for DemoSceneProvider {

    fn scene(&self, options: &HashMap<String, String>) -> Scene {
        let mut scene = Scene::new();

        scene.set_camera(
            Camera::default()
                .with_transform(Transform::new(Vector3::new(0.0, 0.5, 5.0), Vector3::zero()))
        );

        let model_loader: Box<dyn ModelLoader> = Livestonk::resolve();
        let model = model_loader.load("assets/cow.obj").unwrap();
        let cow = PolygonObject::from_model(
            Transform::new(Vector3::new(0.0, 0.32, 8.5), Vector3::new(90.0, 0.0, 0.0)),
            &model
        );
        scene.add_object(box cow);

        let solid_blue = Material::Lambertian {
            albedo: 0.18,
            color: Pixel::from_rgb(13, 71, 161),
        };
        let plane = Plane::new(Transform::default(), solid_blue);
        scene.add_object(box plane);

        /*
        let reflective = Material::Reflective;

        let mut sphere = Sphere::new(3, Transform::new(Vector3::new(0.0, 2.0, 0.0), Vector3::zero()), reflective, 1.0);
        scene.add_object(box sphere);

        let mut another_sphere = Sphere::new(4, Transform::new(Vector3::new(-3.0, 3.0, 2.0), Vector3::zero()), Material::Lambertian {
            albedo: 0.18,
            color: Pixel::from_rgb(13, 71, 161),
        }, 1.0);
        scene.add_object(box another_sphere);*/

        scene.add_light(box PointLight::new(
            Transform::new(
                Vector3::new(0.0, 8.0, 10.0),
                Vector3::new(45.0, -45.0, -70.0)
            ),
            1000.0,
        ));

        scene
    }
}