
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3 { x, y, z }
    }
    pub fn zero() -> Self {
        Vector3 { x: 0.0, y: 0.0, z: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Vertex {
    geometry: Vector3,
    normal: Vector3,
}

impl Vertex {
    pub const fn new(geometry: Vector3, normal: Vector3) -> Self {
        Vertex { geometry, normal, }
    }

    pub fn get_geometry(&self) -> &Vector3 {
        &self.geometry
    }

    pub fn get_normal(&self) -> &Vector3 {
        &self.normal
    }
}

#[derive(Debug, Clone)]
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