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

#[derive(Component)]
pub struct DemoSceneProvider {
    model_loader: Box<dyn ModelLoader>,
}

impl SceneProvider for DemoSceneProvider {

    fn scene(&self, options: &HashMap<String, String>) -> Scene {
        let mut scene = Scene::new();

        // let model = &self.model_loader.load(options.get("source").unwrap_or(&"assets/1.obj".to_string()))
        // let model = &self.model_loader.load(options.get("source").unwrap_or(&"assets/dragon3.obj".to_string()))
        let model = &self.model_loader.load(options.get("source").unwrap_or(&"assets/cow.obj".to_string()))
            .expect("Failed to load model");

        scene.set_camera(
            Camera::default()
                // .with_transform(Transform::new(Vector3::new(0.0, 0.2, 1.5), Vector3::zero()))
                .with_transform(Transform::new(Vector3::new(0.0, 0.9, 1.0), Vector3::new(90.0, 0.0, 90.0)))
        );

        //scene.add_object(box Sphere::new(Transform::new(Vector3::new(0.0, 0.0, -3.0), Vector3::zero()), 1.0));
        scene.add_object(box PolygonObject::from_model(Transform::default(), &model));
        //scene.add_object(box Cube::new(Transform::new(&Vector3::new(0.0, 0.0, -5.0)), 1.0));

        /*scene.add_light(box DistantLight::new(
            Transform::new(
                Vector3::zero(),
                Vector3::new(0.0, -35.0, 0.0)
            ),
            1.0
        ));*/

        scene.add_light(box DistantLight::new(
            Transform::new(
                Vector3::zero(),
                Vector3::new(0.0, -45.0, -45.0)
            ),
            0.4,
        ));

        scene
    }
}