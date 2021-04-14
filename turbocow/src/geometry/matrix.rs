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

    pub fn for_transformation(translation: &Vector3) -> Self {
        Self::empty().apply_translation(translation)
    }

    pub fn apply_translation(&self, translation: &Vector3) -> Matrix44 {
        let mut values = self.values.clone();
        values[0][3] = translation.x;
        values[1][3] = translation.y;
        values[2][3] = translation.z;
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