use livestonk::Component;
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

#[derive(Component)]
pub struct DemoSceneProvider {
    model_loader: Box<dyn ModelLoader>,
}

impl SceneProvider for DemoSceneProvider {

    fn scene(&self, options: &HashMap<String, String>) -> Scene {
        let mut scene = Scene::new();

        scene.set_camera(
            Camera::default()
                .with_transform(Transform::new(Vector3::new(0.0, 1.0, 5.0), Vector3::zero()))
        );

        let plane = Plane::new(Transform::default(), Material::Reflective);
        scene.add_object(box plane);

        let mut sphere = Sphere::new(3, Transform::new(Vector3::new(0.0, 2.0, 0.0), Vector3::zero()), Material::Lambertian {
            albedo: 0.18,
            color: Pixel::from_rgb(13, 71, 161),
        }, 1.0);
        scene.add_object(box sphere);

        let mut another_sphere = Sphere::new(4, Transform::new(Vector3::new(-3.0, 3.0, 2.0), Vector3::zero()), Material::Lambertian {
            albedo: 0.18,
            color: Pixel::from_rgb(13, 71, 161),
        }, 1.0);
        scene.add_object(box another_sphere);

        scene.add_light(box PointLight::new(
            Transform::new(
                Vector3::new(0.0, 4.0, 4.0),
                Vector3::new(45.0, -45.0, -70.0)
            ),
            100.0,
        ));

        scene
    }
}