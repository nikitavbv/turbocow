use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::geometry::models::{Vector3, Vertex, Polygon};
use regex::Regex;

#[derive(Debug)]
pub struct ObjFile {
    vertices: Vec<Vector3>,
    vertices_normals: Vec<Vector3>,
    polygons: Vec<Polygon>,
}

impl ObjFile {
    pub const fn new() -> Self {
        ObjFile {
            vertices: Vec::new(),
            vertices_normals: Vec::new(),
            polygons: Vec::new(),
        }
    }

    pub fn load(&mut self, filename: &str) -> Result<(), ()> {
        let vertex_normal = "vn";
        let vertex = "v";
        let comment = "#";
        let face = "f";

        let file = File::open(filename).unwrap();
        let lines = BufReader::new(file).lines();
        for line in lines {
            let line_data = line.unwrap();
            if line_data.len() == 0 || line_data.starts_with(comment) {
                continue;
            } else if line_data.starts_with(vertex_normal) {
                self.parse_vertex_normal(line_data);
            } else if line_data.starts_with(vertex) {
                self.parse_vertex(line_data);
            } else if line_data.starts_with(face) {
                self.parse_face(line_data);
            } else {
                panic!("Unable to parse line: {}", line_data);
            }
        }
        Result::Ok(())
    }

    fn parse_vertex_normal(&mut self, line: String) {
        let line = Regex::new(r"\\s+").unwrap().replace_all(line.as_str(), " ");
        let mut values = line.split(" ");
        values.next(); // skip 'vn'
        let x = values.next().unwrap().parse::<f64>().unwrap();
        let y = values.next().unwrap().parse::<f64>().unwrap();
        let z = values.next().unwrap().parse::<f64>().unwrap();
        self.vertices_normals.push(Vector3::new(x, y, z));
    }

    fn parse_vertex(&mut self, line: String) {
        let line = Regex::new(r"\\s+").unwrap().replace_all(line.as_str(), " ");
        let mut values = line.split(" ");
        values.next(); // skip 'v'
        let x = values.next().unwrap().parse::<f64>().unwrap();
        let y = values.next().unwrap().parse::<f64>().unwrap();
        let z = values.next().unwrap().parse::<f64>().unwrap();
        self.vertices.push(Vector3::new(x, y, z));
    }

    fn parse_face(&mut self, line: String) {
        let line = Regex::new(r"\\s+").unwrap().replace_all(line.as_str(), " ");
        let mut values = line.split(" ");
        values.next(); // skip 'f'
        let mut vertices = Vec::new();
        for _ in 0..3 {
            let mut value = values.next().unwrap().split("/");
            let vertex_number = value.next().unwrap().parse::<usize>().unwrap();
            if let Some(_) = value.next() {
                if let Some(vn) = value.next() {
                    println!("Why I'm here");
                    let vertex_normal = vn.parse::<usize>().unwrap();
                    vertices.push(Vertex::new(self.vertices[vertex_number - 1], self.vertices_normals[vertex_normal - 1]));
                } else {
                    vertices.push(Vertex::new(self.vertices[vertex_number - 1], Vector3::zero()));
                }
            } else {
                vertices.push(Vertex::new(self.vertices[vertex_number - 1], Vector3::zero()));
            }
        }
        self.polygons.push(Polygon::new(vertices));
    }
}

#[cfg(test)]
mod tests {
    use crate::obj::obj_file_reader::ObjFile;
    #[test]
    fn test1() {
        let mut model = ObjFile::new();
        model.load("./assets/simple.obj").unwrap();
        assert_eq!(model.vertices.len(), 6);
        assert_eq!(model.vertices_normals.len(), 0);
        assert_eq!(model.polygons.len(), 4);
    }
}