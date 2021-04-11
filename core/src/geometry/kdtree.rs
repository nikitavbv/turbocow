use crate::geometry::models::{Vector3, Polygon};

pub struct BoundingBox {
    pub first: Vector3,
    pub second: Vector3,
}

impl BoundingBox {
    pub const fn new(first: Vector3, second: Vector3) -> Self {
        BoundingBox {
            first,
            second,
        }
    }

    pub fn from_polygon(polygon: &Polygon) -> Self {
        let vertices: Vec<&Vector3> = polygon
            .get_vertices()
            .into_iter()
            .map(|x| x.get_geometry())
            .collect();
        println!("from_polygon: vertices: {:?}", vertices);
        let mut min_x = vertices[0].x;
        let mut min_y = vertices[0].y;
        let mut min_z = vertices[0].z;
        let mut max_x = vertices[0].x;
        let mut max_y = vertices[0].y;
        let mut max_z = vertices[0].z;
        let mut iter = vertices.iter();
        iter.next();
        while let Some(vertex) = iter.next() {
            if vertex.x < min_x { min_x = vertex.x }
            if vertex.x > max_x { max_x = vertex.x }
            if vertex.y < min_y { min_y = vertex.y }
            if vertex.y > max_y { max_y = vertex.y }
            if vertex.z < min_z { min_z = vertex.z }
            if vertex.z > max_z { max_z = vertex.z }
        }
        BoundingBox {
            first: Vector3::new(min_x, min_y, min_z),
            second: Vector3::new(max_x, max_y, max_z),
        }
    }

    pub fn extend(&mut self, polygon: &Polygon) {
        let vertices: Vec<&Vector3> = polygon
            .get_vertices()
            .into_iter()
            .map(|x| x.get_geometry())
            .collect();
        println!("extend: vertices: {:?}", vertices.len());
        let mut min_x = self.first.x;
        let mut min_y = self.first.y;
        let mut min_z = self.first.z;
        let mut max_x = self.second.x;
        let mut max_y = self.second.y;
        let mut max_z = self.second.z;
        let mut iter = vertices.iter();
        while let Some(vertex) = iter.next() {
            if vertex.x < min_x { min_x = vertex.x }
            if vertex.x > max_x { max_x = vertex.x }
            if vertex.y < min_y { min_y = vertex.y }
            if vertex.y > max_y { max_y = vertex.y }
            if vertex.z < min_z { min_z = vertex.z }
            if vertex.z > max_z { max_z = vertex.z }
        }
        self.first = Vector3::new(min_x, min_y, min_z);
        self.second = Vector3::new(max_x, max_y, max_z);
    }
}

pub enum NodeType {
    Node(BoundingBox, Box<Vec<NodeType>>),
    LeafNode(BoundingBox, Vec<Polygon>),
    Empty
}

pub struct KDTree {
    node: NodeType
}

enum Side {
    Left,
    Middle,
    Right
}

impl KDTree {
    fn coordinate_fn(axis: u8) -> Box<dyn Fn(&Vector3) -> f64> {
        if axis == 0 {
            Box::new(|vector| vector.x)
        } else if axis == 1 {
            Box::new(|vector| vector.y)
        } else if axis == 2 {
            Box::new(|vector| vector.z)
        } else {
            panic!("unable to split: axis: {}", axis);
        }
    }

    fn determine_side(x1: f64, x2: f64, x3: f64, x: f64) -> Side {
        if x1 < x && x2 < x && x3 < x {
            Side::Left
        } else if x1 > x && x2 > x && x3 > x {
            Side::Right
        } else {
            Side::Middle
        }
    }

    fn split_box(bounding_box: &BoundingBox, polygons: Vec<Polygon>, split_axis: u8) -> Vec<NodeType> {
        let min_x = bounding_box.first.x;
        let max_x = bounding_box.second.x;
        let x = (min_x + max_x) / (2 as f64);
        let get_coorginate = KDTree::coordinate_fn(split_axis);
        let mut left = Vec::new();
        let mut middle = Vec::new();
        let mut right = Vec::new();
        for polygon in polygons.into_iter() {
            match KDTree::determine_side(
                get_coorginate(polygon.get_vertices()[0].get_geometry()),
                get_coorginate(polygon.get_vertices()[1].get_geometry()),
                get_coorginate(polygon.get_vertices()[2].get_geometry()),
                x
            ) {
                Side::Left => left.push(polygon),
                Side::Middle => middle.push(polygon),
                Side::Right => right.push(polygon),
            };
        }
        vec![
            NodeType::LeafNode(calculate_bounding_box(&left), left),
            NodeType::LeafNode(calculate_bounding_box(&middle), middle),
            NodeType::LeafNode(calculate_bounding_box(&right), right)
        ]
    }

    fn build_tree(node: NodeType, mut split_axis: u8) -> NodeType {
        match node {
            NodeType::Node(_, _) => node,
            NodeType::LeafNode(bounding_box, polygons) => {
                if polygons.len() < 8 {
                    NodeType::LeafNode(bounding_box, polygons)
                } else {
                    let childs = KDTree::split_box(&bounding_box, polygons, split_axis)
                        .into_iter();
                    split_axis = (split_axis + 1) % 3;
                    let childs = childs.map(|child| KDTree::build_tree(child, split_axis))
                        .collect();
                    NodeType::Node(bounding_box, Box::new(childs))
                }
            },
            NodeType::Empty => node,
        }
    }

    pub fn build(bounding_box: BoundingBox, polygons: Vec<Polygon>) -> Self {
        KDTree {
            node: KDTree::build_tree(NodeType::LeafNode(bounding_box, polygons), 0)
        }
    }
}

fn calculate_bounding_box(polygons: &Vec<Polygon>) -> BoundingBox {
    let mut bounding_box = BoundingBox::from_polygon(&polygons[0]);
    for i in 1..polygons.len() {
        bounding_box.extend(&polygons[i]);
    }
    bounding_box
}

pub fn build_tree(polygons: Vec<Polygon>) -> KDTree {
    KDTree::build(calculate_bounding_box(&polygons), polygons)
}

#[cfg(test)]
mod tests {
    use crate::geometry::models::{Vector3, Vertex, Polygon};
    use crate::geometry::kdtree::BoundingBox;

    #[test]
    fn test_bounding_box_from_polygon() {
        let v1 = Vertex::new(Vector3::new(2.6, -3.0, 2.0), Vector3::zero());
        let v2 = Vertex::new(Vector3::new(1.3, 1.5, 2.9), Vector3::zero());
        let v3 = Vertex::new(Vector3::new(-0.8, 0.6, 3.3), Vector3::zero());
        let polygon = Polygon::new(vec![v1, v2, v3]);
        let bounding_box = BoundingBox::from_polygon(&polygon);
        assert_eq!(bounding_box.first, Vector3::new(-0.8, -3.0, 2.0));
        assert_eq!(bounding_box.second, Vector3::new(2.6, 1.5, 3.3));
    }

    #[test]
    fn test_bounding_box_extend1() {
        let v1 = Vertex::new(Vector3::new(2.6, -3.0, 2.0), Vector3::zero());
        let v2 = Vertex::new(Vector3::new(1.3, 1.5, 2.9), Vector3::zero());
        let v3 = Vertex::new(Vector3::new(-0.8, 0.6, 3.3), Vector3::zero());
        let polygon = Polygon::new(vec![v1, v2, v3]);
        let mut bounding_box = BoundingBox::from_polygon(&polygon);
        
        let v1 = Vertex::new(Vector3::new(1.8, -3.5, 2.0), Vector3::zero());
        let v2 = Vertex::new(Vector3::new(1.3, 1.6, 1.1), Vector3::zero());
        let v3 = Vertex::new(Vector3::new(-0.4, 0.5, 3.15), Vector3::zero());
        let polygon = Polygon::new(vec![v1, v2, v3]);
        bounding_box.extend(&polygon);
        assert_eq!(bounding_box.first, Vector3::new(-0.8, -3.5, 1.1));
        assert_eq!(bounding_box.second, Vector3::new(2.6, 1.6, 3.3));
    }

    #[test]
    fn test_bounding_box_extend2() {
        let v1 = Vertex::new(Vector3::new(2.6, -3.0, 2.0), Vector3::zero());
        let v2 = Vertex::new(Vector3::new(1.3, 1.5, 2.9), Vector3::zero());
        let v3 = Vertex::new(Vector3::new(-0.8, 0.6, 3.3), Vector3::zero());
        let polygon = Polygon::new(vec![v1, v2, v3]);
        let mut bounding_box = BoundingBox::from_polygon(&polygon);
        
        let v1 = Vertex::new(Vector3::new(2.0, -2.8, 2.2), Vector3::zero());
        let v2 = Vertex::new(Vector3::new(0.9, 0.75, 3.0), Vector3::zero());
        let v3 = Vertex::new(Vector3::new(-0.5, 0.1, 2.5), Vector3::zero());
        let polygon = Polygon::new(vec![v1, v2, v3]);
        bounding_box.extend(&polygon);
        assert_eq!(bounding_box.first, Vector3::new(-0.8, -3.0, 2.0));
        assert_eq!(bounding_box.second, Vector3::new(2.6, 1.5, 3.3));
    }
}
