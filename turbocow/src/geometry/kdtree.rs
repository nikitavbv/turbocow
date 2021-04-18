use crate::geometry::vector3::Vector3;
use crate::geometry::ray::Ray;
use crate::objects::triangle::Triangle;

const DELTA: f64 = 1e-6;

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

    pub const fn empty() -> Self {
        BoundingBox {
            first: Vector3::new(0.0, 0.0, 0.0),
            second: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn from_triangle(triangle: &Triangle) -> Self {
        let vertices = triangle.get_vertices();
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

    pub fn extend(&mut self, triangle: &Triangle) {
        let vertices = triangle.get_vertices();
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

    pub fn sqare(&self) -> f64 {
        let x = (self.second.x - self.first.x).abs();
        let y = (self.second.y - self.first.y).abs();
        let z = (self.second.z - self.first.z).abs();
        x * y + y * x + x * z
    }
}

pub enum NodeType {
    Node(BoundingBox, Box<Vec<NodeType>>),
    LeafNode(BoundingBox, Vec<Triangle>),
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
        } else {
            Box::new(|vector| vector.z)
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

    fn calculate_sah(bounding_box: &BoundingBox, triangles: &Vec<Triangle>, split_axis: u8, v: f64) -> (f64, u8) {
        let get_coorginate = KDTree::coordinate_fn(split_axis);
        let mut left_triangles = 0;
        let mut left_box = BoundingBox::empty();
        let mut middle_triangles = 0;
        let mut middle_box = BoundingBox::empty();
        let mut right_triangles = 0;
        let mut right_box = BoundingBox::empty();
        for triangle in triangles.iter() {
            match KDTree::determine_side(
                get_coorginate(triangle.get_vertices()[0]),
                get_coorginate(triangle.get_vertices()[1]),
                get_coorginate(triangle.get_vertices()[2]),
                v
            ) {
                Side::Left => {
                    left_triangles += 1;
                    left_box.extend(triangle);
                },
                Side::Middle => {
                    middle_triangles += 1;
                    middle_box.extend(triangle);
                },
                Side::Right => {
                    right_triangles += 1;
                    right_box.extend(triangle);
                },
            };
        }
        // println!("triangles: {} {} {}", left_triangles, middle_triangles, right_triangles);
        let mut sah_value = 0.0;
        let mut nodes = 0;
        if left_triangles > 0 {
            sah_value += left_triangles as f64 * left_box.sqare();
            nodes += 1;
        }
        if middle_triangles > 0 {
            sah_value += middle_triangles as f64 * middle_box.sqare();
            nodes += 1;
        }
        if right_triangles > 0 {
            sah_value += right_triangles as f64 * right_box.sqare();
            nodes += 1;
        }
        // println!("sah: {}", sah_value);
        (sah_value, nodes)
    }

    fn split_box_sah(bounding_box: &BoundingBox, triangles: Vec<Triangle>, split_axis: u8) -> Vec<NodeType> {
        // println!("split box sah");
        let get_coorginate = KDTree::coordinate_fn(split_axis);
        let min = get_coorginate(&bounding_box.first);
        let max = get_coorginate(&bounding_box.second);
        let step = (max - min) / 16.0;
        let mut i = min + step;
        let (mut sah_value, mut nodes) = KDTree::calculate_sah(bounding_box, &triangles, split_axis, i);
        let mut split_coordinate = i;
        i += step;
        while i < max {
            let (tmp_sah, tmp_nodes) = KDTree::calculate_sah(bounding_box, &triangles, split_axis, i);
            if (nodes == 1 && tmp_nodes > 1) || (tmp_sah < sah_value && tmp_nodes >= nodes) {
                sah_value = tmp_sah;
                split_coordinate = i;
                nodes = tmp_nodes;
            }
            i += step;
        }
        KDTree::split_box_by_plane(triangles, split_axis, split_coordinate)
    }

    fn split_box_by_plane(triangles: Vec<Triangle>, split_axis: u8, v: f64) -> Vec<NodeType> {
        let get_coorginate = KDTree::coordinate_fn(split_axis);
        let mut left = Vec::new();
        let mut middle = Vec::new();
        let mut right = Vec::new();
        for triangle in triangles.into_iter() {
            match KDTree::determine_side(
                get_coorginate(triangle.get_vertices()[0]),
                get_coorginate(triangle.get_vertices()[1]),
                get_coorginate(triangle.get_vertices()[2]),
                v
            ) {
                Side::Left => left.push(triangle),
                Side::Middle => middle.push(triangle),
                Side::Right => right.push(triangle),
            };
        }
        // println!("{} {} {}", left.len(), middle.len(), right.len());
        let mut nodes = Vec::new();
        if left.len() > 0 {
            nodes.push(NodeType::LeafNode(calculate_bounding_box(&left), left));
        }
        if middle.len() > 0 {
            nodes.push(NodeType::LeafNode(calculate_bounding_box(&middle), middle));
        }
        if right.len() > 0 {
            nodes.push(NodeType::LeafNode(calculate_bounding_box(&right), right));
        }
        // println!("split_box_by_plane: nodes.len: {}", nodes.len());
        nodes
    }

    fn split_box(bounding_box: &BoundingBox, triangles: Vec<Triangle>, split_axis: u8) -> Vec<NodeType> {
        let get_coorginate = KDTree::coordinate_fn(split_axis);
        let min = get_coorginate(&bounding_box.first);
        let max = get_coorginate(&bounding_box.second);
        let v = (min + max) / (2 as f64);
        KDTree::split_box_by_plane(triangles, split_axis, v)
    }

    fn build_tree(node: NodeType, mut split_axis: u8) -> NodeType {
        // println!("build tree fn");
        match node {
            NodeType::Node(_, _) => node,
            NodeType::LeafNode(bounding_box, triangles) => {
                // println!("triangles.len(): {}", triangles.len());
                if triangles.len() <= 8 {
                    NodeType::LeafNode(bounding_box, triangles)
                } else {
                    // let childs = KDTree::split_box(&bounding_box, triangles, split_axis);
                    let childs = KDTree::split_box_sah(&bounding_box, triangles, split_axis);
                    // println!("childs.len(): {}", childs.len());
                    let childs = childs.into_iter();
                    split_axis = (split_axis + 1) % 3;
                    let childs = childs.map(|child| KDTree::build_tree(child, split_axis))
                        .collect();
                    NodeType::Node(bounding_box, Box::new(childs))
                }
            },
        }
    }

    pub fn build(bounding_box: BoundingBox, triangles: Vec<Triangle>) -> Self {
        KDTree {
            node: KDTree::build_tree(NodeType::LeafNode(bounding_box, triangles), 0)
        }
    }

    fn check_intersection(&self, bounding_box: &BoundingBox, ray: &Ray) -> bool {
        let direction = ray.direction();
        let origin = ray.origin();
        let x = axis_direction(direction.x);
        let y = axis_direction(direction.y);
        let z = axis_direction(direction.z);
        let lower_bounds = bounding_box.first;
        let upper_bounds = bounding_box.second;

        let (t_min, t_max) = if direction.x >= 0.0 {
            (
                (lower_bounds.x - origin.x) * x,
                (upper_bounds.x - origin.x) * x,
            )
        } else {
            (
                (upper_bounds.x - origin.x) * x,
                (lower_bounds.x - origin.x) * x,
            )
        };

        let (ty_min, ty_max) = if direction.y >= 0.0 {
            (
                (lower_bounds.y - origin.y) * y,
                (upper_bounds.y - origin.y) * y,
            )
        } else {
            (
                (upper_bounds.y - origin.y) * y,
                (lower_bounds.y - origin.y) * y,
            )
        };

        if t_min > ty_max || ty_min > t_max {
            return false;
        }
        
        let t_min = ty_min.max(t_min);
        let t_max = ty_max.min(t_max);

        let (tz_min, tz_max) = if direction.z >= 0.0 {
            (
                (lower_bounds.z - origin.z) * z,
                (upper_bounds.z - origin.z) * z,
            )
        } else {
            (
                (upper_bounds.z - origin.z) * z,
                (lower_bounds.z - origin.z) * z,
            )
        };

        if t_min > tz_max || tz_min > t_max {
            return false;
        }

        let t_min = tz_min.max(t_min);
        let t_max = tz_max.min(t_max);

        if t_min < 0.0 && t_max < 0.0 {
            return false;
        }
        true
    }

    fn find_triangles(&self, node: &NodeType, ray: &Ray) -> Vec<Triangle> {
        match node {
            NodeType::Node(bounding_box, nodes) => {
                if self.check_intersection(bounding_box, ray) {
                    let mut triangles = Vec::new();
                    for node in nodes.iter() {
                        triangles.extend_from_slice(&self.find_triangles(node, ray)[0..]);
                    }
                    triangles
                } else {
                    Vec::new()
                }
            },
            NodeType::LeafNode(bounding_box, triangles) => {
                if self.check_intersection(bounding_box, ray) {
                    triangles.clone()
                } else {
                    Vec::new()
                }
            },
        }
    }

    pub fn get_triangles(&self, ray: &Ray) -> Vec<Triangle> {
        self.find_triangles(&self.node, ray)
    }
}

fn axis_direction(val: f64) -> f64 {
    if val.abs() < DELTA {
        if val >= 0.0 {
            1.0 / DELTA
        } else {
            -1.0 / DELTA
        }
    } else {
        1.0 / val
    }
}

fn calculate_bounding_box(triangles: &Vec<Triangle>) -> BoundingBox {
    let mut bounding_box = BoundingBox::from_triangle(&triangles[0]);
    for i in 1..triangles.len() {
        bounding_box.extend(&triangles[i]);
    }
    bounding_box
}

pub fn build_tree(triangles: Vec<Triangle>) -> KDTree {
    KDTree::build(calculate_bounding_box(&triangles), triangles)
}

#[cfg(test)]
mod tests {
    use crate::geometry::vector3::Vector3;
    use crate::geometry::kdtree::BoundingBox;
    use crate::objects::triangle::Triangle;
    use crate::geometry::transform::Transform;

    #[test]
    fn test_bounding_box_from_polygon() {
        let v1 = Vector3::new(2.6, -3.0, 2.0);
        let v2 = Vector3::new(1.3, 1.5, 2.9);
        let v3 = Vector3::new(-0.8, 0.6, 3.3);
        let triangle = Triangle::new(Transform::default(), v1, v2, v3);
        let bounding_box = BoundingBox::from_triangle(&triangle);
        assert_eq!(bounding_box.first, Vector3::new(-0.8, -3.0, 2.0));
        assert_eq!(bounding_box.second, Vector3::new(2.6, 1.5, 3.3));
    }

    #[test]
    fn test_bounding_box_extend1() {
        let v1 = Vector3::new(2.6, -3.0, 2.0);
        let v2 = Vector3::new(1.3, 1.5, 2.9);
        let v3 = Vector3::new(-0.8, 0.6, 3.3);
        let triangle = Triangle::new(Transform::default(), v1, v2, v3);
        let mut bounding_box = BoundingBox::from_triangle(&triangle);
        
        let v1 = Vector3::new(1.8, -3.5, 2.0);
        let v2 = Vector3::new(1.3, 1.6, 1.1);
        let v3 = Vector3::new(-0.4, 0.5, 3.15);
        let triangle = Triangle::new(Transform::default(), v1, v2, v3);
        bounding_box.extend(&triangle);
        assert_eq!(bounding_box.first, Vector3::new(-0.8, -3.5, 1.1));
        assert_eq!(bounding_box.second, Vector3::new(2.6, 1.6, 3.3));
    }

    #[test]
    fn test_bounding_box_extend2() {
        let v1 = Vector3::new(2.6, -3.0, 2.0);
        let v2 = Vector3::new(1.3, 1.5, 2.9);
        let v3 = Vector3::new(-0.8, 0.6, 3.3);
        let triangle = Triangle::new(Transform::default(), v1, v2, v3);
        let mut bounding_box = BoundingBox::from_triangle(&triangle);
        
        let v1 = Vector3::new(2.0, -2.8, 2.2);
        let v2 = Vector3::new(0.9, 0.75, 3.0);
        let v3 = Vector3::new(-0.5, 0.1, 2.5);
        let triangle = Triangle::new(Transform::default(), v1, v2, v3);
        bounding_box.extend(&triangle);
        assert_eq!(bounding_box.first, Vector3::new(-0.8, -3.0, 2.0));
        assert_eq!(bounding_box.second, Vector3::new(2.6, 1.5, 3.3));
    }
}
