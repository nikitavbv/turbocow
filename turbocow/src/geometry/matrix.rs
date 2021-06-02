use super::vector3::Vector3;

#[derive(Clone)]
pub struct Matrix44 {

    values: [[f64; 4]; 4],
}

impl Matrix44 {

    pub fn new(values: [[f64; 4]; 4]) -> Self {
        Matrix44 { 
            values 
        }
    }

    pub fn empty() -> Self {
        let mut values = [[0f64; 4]; 4];
        for i in 0..4 {
            values[i][i] = 1.0;
        }

        Self::new(values)
    }

    pub fn for_transformation(translation: &Vector3, rotation: &Vector3) -> Self {
        Self::empty()
            .apply_translation(translation)
            //.apply_rotation(rotation)
    }

    pub fn apply_translation(&self, translation: &Vector3) -> Matrix44 {
        let mut values = self.values.clone();
        values[0][3] = translation.x;
        values[1][3] = translation.y;
        values[2][3] = translation.z;
        Matrix44::new(values)
    }

    pub fn apply_rotation(&self, rotation: &Vector3) -> Matrix44 {
        let rot = apply_rotation_around_x(rotation.x);
        let rot = multiply(rot, apply_rotation_around_y(rotation.y));
        let rot = multiply(rot, apply_rotation_around_z(rotation.z));

        let mut values = self.values.clone();
        for i in 0..3 {
            for j in 0..3 {
                values[j][i] = rot[j][i];
            }
        }

        Matrix44::new(values)
    }

    pub fn apply_for_point(&self, point: &Vector3) -> Vector3 {
        self.translate(&self.apply_for_vector(point))
    }

    pub fn apply_for_vector(&self, vector: &Vector3) -> Vector3 {
        Vector3::new(
            vector.x * self.values[0][0] + vector.y * self.values[0][1] + vector.z * self.values[0][2],
            vector.x * self.values[1][0] + vector.y * self.values[1][1] + vector.z * self.values[1][2],
            vector.x * self.values[2][0] + vector.y * self.values[2][1] + vector.z * self.values[2][2]
        )
    }

    pub fn translate(&self, vector: &Vector3) -> Vector3 {
        vector + &Vector3::new(self.values[0][3], self.values[1][3], self.values[2][3])
    }
}

fn apply_rotation_around_x(angle: f64) -> [[f64; 3]; 3] {
    let angle = angle.to_radians();

    let mut rotation = [[0f64; 3]; 3];
    rotation[0][0] = 1.0;
    rotation[1][1] = angle.cos();
    rotation[1][2] = angle.sin();
    rotation[2][1] = -angle.sin();
    rotation[2][2] = angle.cos();

    rotation
}

fn apply_rotation_around_y(angle: f64) -> [[f64; 3]; 3] {
    let angle = angle.to_radians();

    let mut rotation = [[0f64; 3]; 3];
    rotation[1][1] = 1.0;
    rotation[0][0] = angle.cos();
    rotation[0][2] = -angle.sin();
    rotation[2][0] = angle.sin();
    rotation[2][2] = angle.cos();

    rotation
}

fn apply_rotation_around_z(angle: f64) -> [[f64; 3]; 3] {
    let angle = angle.to_radians();

    let mut rotation = [[0f64; 3]; 3];
    rotation[2][2] = 1.0;
    rotation[0][0] = angle.cos();
    rotation[0][1] = angle.sin();
    rotation[1][0] = -angle.sin();
    rotation[1][1] = angle.cos();

    rotation
}

fn multiply(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut result = [[0f64; 3]; 3];

    for i in 0..3 {
        for j in 0..3 {
            result[i][j] = a[i][0] * b[0][j]
                + a[i][1] * b[1][j]
                + a[i][2] * b[2][j];
        }
    }

    result
}