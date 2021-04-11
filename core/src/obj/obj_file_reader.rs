use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::geometry::models::{Vector3, Vertex, Polygon};
use regex::Regex;
use custom_error::custom_error;

custom_error! {pub ObjFileError
    VertexError {description: String} = "Failed to parse vertex: {description}",
    VertexNormalError {description: String} = "Failed to parse vertex normal: {description}",
    FaceError {description: String} = "Failed to parse face: {description}",
}

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

    pub fn load(&mut self, filename: &str) -> Result<(), ObjFileError> {
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
                self.parse_vertex_normal(line_data)?;
            } else if line_data.starts_with(vertex) {
                self.parse_vertex(line_data)?;
            } else if line_data.starts_with(face) {
                self.parse_face(line_data)?;
            } else {
                panic!("Unable to parse line: {}", line_data);
            }
        }
        Result::Ok(())
    }

    fn parse_vertex_normal(&mut self, line: String) -> Result<(), ObjFileError> {
        let line = Regex::new(r"\\s+").unwrap().replace_all(line.as_str(), " ");
        let mut values = line.split(" ");
        values.next(); // skip 'vn'
        let x = values.next();
        if x.is_none() {
            return Result::Err(ObjFileError::VertexNormalError {
                description: format!("Unable to parse first coordinate: {}", &line)
            });
        }
        let x = x.unwrap().parse::<f64>().map_err(|err| ObjFileError::VertexNormalError {
            description: format!("Unable to parse first coordinate: {}. Cause: {:?}", &line, err)
        })?;
        let y = values.next();
        if y.is_none() {
            return Result::Err(ObjFileError::VertexNormalError {
                description: format!("Unable to parse second coordinate: {}", &line)
            });
        }
        let y = y.unwrap().parse::<f64>().map_err(|err| ObjFileError::VertexNormalError {
            description: format!("Unable to parse second coordinate: {}. Cause: {:?}", &line, err)
        })?;
        let z = values.next();
        if z.is_none() {
            return Result::Err(ObjFileError::VertexNormalError {
                description: format!("Unable to parse third coordinate: {}", &line)
            });
        }
        let z = z.unwrap().parse::<f64>().map_err(|err| ObjFileError::VertexNormalError {
            description: format!("Unable to parse third coordinate: {}. Cause: {:?}", &line, err)
        })?;
        self.vertices_normals.push(Vector3::new(x, y, z));
        Result::Ok(())
    }

    fn parse_vertex(&mut self, line: String) -> Result<(), ObjFileError> {
        let line = Regex::new(r"\\s+").unwrap().replace_all(line.as_str(), " ");
        let mut values = line.split(" ");
        values.next(); // skip 'v'
        let x = values.next();
        if x.is_none() {
            return Result::Err(ObjFileError::VertexNormalError {
                description: format!("Unable to parse first coordinate: {}", &line)
            });
        }
        let x = x.unwrap().parse::<f64>().map_err(|err| ObjFileError::VertexNormalError {
            description: format!("Unable to parse first coordinate: {}. Cause: {:?}", &line, err)
        })?;
        let y = values.next();
        if y.is_none() {
            return Result::Err(ObjFileError::VertexNormalError {
                description: format!("Unable to parse second coordinate: {}", &line)
            });
        }
        let y = y.unwrap().parse::<f64>().map_err(|err| ObjFileError::VertexNormalError {
            description: format!("Unable to parse second coordinate: {}. Cause: {:?}", &line, err)
        })?;
        let z = values.next();
        if z.is_none() {
            return Result::Err(ObjFileError::VertexNormalError {
                description: format!("Unable to parse third coordinate: {}", &line)
            });
        }
        let z = z.unwrap().parse::<f64>().map_err(|err| ObjFileError::VertexNormalError {
            description: format!("Unable to parse third coordinate: {}. Cause: {:?}", &line, err)
        })?;
        self.vertices.push(Vector3::new(x, y, z));
        Result::Ok(())
    }

    fn parse_face(&mut self, line: String) -> Result<(), ObjFileError> {
        let line = Regex::new(r"\\s+").unwrap().replace_all(line.as_str(), " ");
        let mut values = line.split(" ");
        values.next(); // skip 'f'
        let mut vertices = Vec::new();
        for _ in 0..3 {
            let mut value = values.next().unwrap().split("/");
            let vertex_number = value.next().unwrap().parse::<usize>().map_err(|err| ObjFileError::FaceError {
                description: format!("Unable to parse vertex number for face: {}. Cause: {:?}", line, err)
            })?;
            if let Some(_) = value.next() {
                if let Some(vn) = value.next() {
                    let vertex_normal = vn.parse::<usize>().map_err(|err| ObjFileError::FaceError {
                        description: format!("Unable to parse vertex normal number for face: {}. Cause: {:?}", line, err)
                    })?;
                    vertices.push(Vertex::new(self.vertices[vertex_number - 1], self.vertices_normals[vertex_normal - 1]));
                } else {
                    vertices.push(Vertex::new(self.vertices[vertex_number - 1], Vector3::zero()));
                }
            } else {
                vertices.push(Vertex::new(self.vertices[vertex_number - 1], Vector3::zero()));
            }
        }
        self.polygons.push(Polygon::new(vertices));
        Result::Ok(())
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
        println!("{:?}", model.polygons);
    }

    #[test]
    fn test2() {
        let mut model = ObjFile::new();
        let res = model.load("./assets/broken.obj");
        match res {
            Ok(_) => panic!("Test should fail due to bad input file!"),
            Err(err) => {
                assert_eq!(format!("{:?}", err), "VertexNormalError { description: \"Unable to parse first coordinate: v 2.292fw449 -0.871852 -0.882400. Cause: ParseFloatError { kind: Invalid }\" }");
            },
        };
    }
}