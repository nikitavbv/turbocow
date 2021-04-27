use std::fs::File;
use std::io::{BufReader, BufRead};

use livestonk::Component;

use crate::geometry::models::{Vertex, Polygon};
use crate::geometry::vector3::Vector3;
use custom_error::custom_error;
use crate::io::traits::{ModelLoader, Model, TurbocowIOError};

custom_error! {pub ObjFileError
    ParseError {description: String} = "Failed to parse line: {description}",
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
}

impl Model for ObjFile {

    fn polygons(&self) -> &Vec<Polygon> {
        &self.polygons
    }
}

#[derive(Component)]
pub(crate) struct ObjFileLoader {
}

impl ObjFileLoader {

    fn new() -> Self {
        Self {
        }
    }

    fn load(obj_file: &mut ObjFile, filename: &str) -> Result<(), ObjFileError> {
        let vertex_normal = "vn";
        let vertex = "v";
        let comment = "#";
        let face = "f";
        let group = "g";
        let object = "o";
        let material = "usemtl";
        let external_material = "mtllib";
        let smooth = "s";

        let file = File::open(filename).unwrap();
        let lines = BufReader::new(file).lines();
        for line in lines {
            let line_data = line.unwrap();
            if line_data.len() == 0 || line_data.starts_with(comment) {
                continue;
            } else if line_data.starts_with(vertex_normal) {
                Self::parse_vertex_normal(obj_file, line_data)?;
            } else if line_data.starts_with(vertex) {
                Self::parse_vertex(obj_file, line_data)?;
            } else if line_data.starts_with(face) {
                Self::parse_face(obj_file, line_data)?;
            } else if line_data.starts_with(group) {
                log:: trace!("group name(s): {}", &line_data[2..]);
            } else if line_data.starts_with(material) {
                log:: trace!("material name(s): {}", &line_data[7..]);
            } else if line_data.starts_with(object) {
                log:: trace!("object name(s): {}", &line_data[2..]);
            } else if line_data.starts_with(smooth) {
                log:: trace!("smooth: {}", &line_data[2..]);
            } else if line_data.starts_with(external_material) {
                log:: trace!("external material name(s): {}", &line_data[7..]);
            } else {
                return Result::Err(ObjFileError::ParseError {
                    description: format!("{}", line_data)
                });
            }
        }
        Result::Ok(())
    }

    pub fn split_line(str: &str) -> Vec<String> {
        let mut res = Vec::new();
        let mut value = "".to_owned();
        for c in str.chars() {
            match c {
                ' ' | '\t' => {
                    if value.len() > 0 {
                        res.push(value);
                    }
                    value = "".to_owned();
                },
                _ => value.push(c),
            };
        }
        if value.len() > 0 {
            res.push(value);
        }
        res
    }

    fn parse_vertex_normal(file: &mut ObjFile, line: String) -> Result<(), ObjFileError> {
        let values = Self::split_line(&line[2..]);
        let mut values = values.iter();
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
        file.vertices_normals.push(Vector3::new(x, y, z));
        Result::Ok(())
    }

    fn parse_vertex(file: &mut ObjFile, line: String) -> Result<(), ObjFileError> {
        let values = Self::split_line(&line[1..]);
        let mut values = values.iter();
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
        file.vertices.push(Vector3::new(x, y, z));
        Result::Ok(())
    }

    fn parse_face(file: &mut ObjFile, line: String) -> Result<(), ObjFileError> {
        let values = Self::split_line(&line[1..]);
        let mut values = values.iter();
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
                    vertices.push(Vertex::new(file.vertices[vertex_number - 1], file.vertices_normals[vertex_normal - 1]));
                } else {
                    vertices.push(Vertex::new(file.vertices[vertex_number - 1], Vector3::zero()));
                }
            } else {
                vertices.push(Vertex::new(file.vertices[vertex_number - 1], Vector3::zero()));
            }
        }
        file.polygons.push(Polygon::new(vertices));
        Result::Ok(())
    }
}

impl ModelLoader for ObjFileLoader {

    fn load(&self, path: &str) -> Result<Box<dyn Model>, TurbocowIOError> {
        let mut file = ObjFile::new();
        Self::load(&mut file,path)
            .map(|_| box file as Box<dyn Model>)
            .map_err(|err| TurbocowIOError::FailedToLoad {
                description: format!("obj file error: {}", err)
            })
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn test_ok() {
        let mut model = ObjFileLoader::new().load("./assets/simple.obj").unwrap();
        //assert_eq!(model.vertices().len(), 6);
        //assert_eq!(model.vertices_normals().len(), 0);
        assert_eq!(model.polygons().len(), 4);
        println!("{:?}", model.polygons());
    }

    #[test]
    fn test_err() {
        let res = ObjFileLoader::new().load("./assets/broken.obj");
        match res {
            Ok(_) => panic!("Test should fail due to bad input file!"),
            Err(err) => {
                assert_eq!(format!("{:?}", err), "FailedToLoad { description: \"obj file error: Failed to parse vertex normal: Unable to parse first coordinate: v 2.292fw449 -0.871852 -0.882400. Cause: ParseFloatError { kind: Invalid }\" }");
            },
        };
    }

    #[test]
    fn test_split() {
        assert_eq!(vec!["-4.43".to_owned(), "0.43".to_owned(), "3".to_owned()], ObjFileLoader::split_line(" -4.43 0.43 3  "));
    }

    #[test]
    fn test_cow() {
        let model = ObjFileLoader::new().load("./assets/dragon3.obj").unwrap();
    }
}
