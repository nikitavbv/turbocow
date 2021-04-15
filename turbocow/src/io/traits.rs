use custom_error::custom_error;
use crate::geometry::models::Polygon;

custom_error! {pub TurbocowIOError
    FailedToLoad {description: String} = "Failed to load model: {description}",
}

pub trait ModelLoader {

    fn load(&self, path: &str) -> Result<Box<dyn Model>, TurbocowIOError>;
}

pub trait Model {

    fn polygons(&self) -> &Vec<Polygon>;
}