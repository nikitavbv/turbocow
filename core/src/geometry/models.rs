use super::vector3::Vector3;

#[derive(Debug)]
pub struct Vertex {
    geometry: Vector3,
    normal: Vector3,
}

impl Vertex {

    pub const fn new(geometry: Vector3, normal: Vector3) -> Self {
        Vertex { geometry, normal, }
    }

    pub fn geometry(&self) -> &Vector3 {
        &self.geometry
    }
}

#[derive(Debug)]
pub struct Polygon {
    vertices: Vec<Vertex>,
}

impl Polygon {
    pub const fn new(vertices: Vec<Vertex>) -> Self {
        Polygon { vertices }
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }
}