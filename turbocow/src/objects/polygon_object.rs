use crate::{geometry::models::Polygon, geometry::transform::Transform, render::intersection::Intersection, geometry::ray::Ray, scene::scene_object::SceneObject, geometry::kdtree::KDTree, geometry::kdtree::build_tree};
use super::triangle::Triangle;
use crate::geometry::vector3::Vector3;
use std::collections::HashMap;
use crate::geometry::models::Vertex;

pub struct PolygonObject {
    id: usize,
    transform: Transform,
    kd_tree: KDTree,
}

impl PolygonObject {

    pub fn from_triangles(id: usize, transform: Transform, triangles: Vec<Triangle>) -> Self {
        Self {
            id,
            transform,
            kd_tree: build_tree(triangles)
        }
    }

    pub fn from_model(id: usize, transform: Transform, file: &sceneformat::MeshGeometry) -> Self {
        Self::from_polygons(id, transform, &polygons_from_file(file))
    }

    pub fn from_polygons(id: usize, transform: Transform, polygons: &Vec<Polygon>) -> Self {
        let mut triangles = Vec::new();

        for polygon in polygons {
            let ver = polygon.get_vertices();
            let pillar = ver[0].geometry();
            let pillar_normal = ver[0].normal();

            for i in 1..ver.len() - 1 {
                triangles.push(Triangle::new_with_normals(
                    transform.clone(),
                    pillar.clone(), 
                    ver[i].geometry().clone(),
                    ver[i + 1].geometry().clone(),
                    pillar_normal.clone(),
                    ver[i].normal().clone(),
                    ver[i + 1].normal().clone()
                ));
            }
        }

        Self::from_triangles(id, transform, triangles)
    }
}

impl SceneObject for PolygonObject {

    fn id(&self) -> usize {
        self.id
    }

    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn check_intersection(&self, ray: &Ray) -> Option<Intersection> {
        let mut min_distance = f64::MAX;
        let mut best_intersection = None;
        let triangles = self.kd_tree.get_triangles(ray);
        for triangle in triangles {
            if let Some(intersection) = triangle.check_intersection(ray) {
                let intersection_distance = intersection.ray_distance();
                
                if intersection_distance < min_distance {
                    min_distance = intersection_distance;
                    best_intersection = Some(intersection);
                }
            }
        }

        best_intersection
    }
}

fn polygons_from_file(meshed_object: &sceneformat::MeshGeometry) -> Vec<Polygon> {
    let mut polygons = Vec::new();

    for face in &meshed_object.faces {
        let mut vertices = Vec::new();

        for element in &face.elements {
            let geometry = meshed_object.vertices.get(element.vertex_index as usize - 1)
                .map(|v| Vector3::new(v.x, v.y, v.z))
                .expect("expected referenced vertex to be present");
            let normal = meshed_object.vertex_normals.get(element.normal_index as usize - 1)
                .map(|v| Vector3::new(v.x, v.y, v.z))
                .expect("expected references normal to be present");
            vertices.push(Vertex::new(geometry, normal));
        }

        polygons.push(Polygon::new(vertices))
    }

    polygons
}