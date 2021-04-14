use std::collections::HashMap;

use custom_error::custom_error;

use super::image::Image;

custom_error! {pub ImageIOError
    FailedToRead {description: String} = "Failed to read image: {description}",
    FailedToWrite {description: String} = "Failed to write image: {description}",
    InvalidOptions {description: String} = "Invalid options are set for this io operation: {description}",
}

pub trait ImageReader {

    fn read(&self, data: &Vec<u8>) -> Result<Vec<Image>, ImageIOError>;
}

pub trait ImageWriter {

    fn write(&self, image: &Image, options: &ImageWriterOptions) -> Result<Vec<u8>, ImageIOError>;
}

pub struct ImageWriterOptions {

    options: HashMap<String, String>,
}

impl ImageWriterOptions {
    
    pub fn default() -> Self {
        Self {
            options: HashMap::new(),
        }
    }

    pub fn with_option(&self, key: &str, value: &str) -> Self {
        let mut options = self.options.clone();
        options.insert(key.to_string(), value.to_string());

        Self {
            options,
        }
    }

    pub fn with_option_u32(&self, key: &str, value: u32) -> Self {
        self.with_option(&key, &value.to_string())
    }

    pub fn with_option_bool(&self, key: &str, value: bool) -> Self {
        self.with_option(&key, if value {
            "true"
        } else {
            "false"
        })
    }

    pub fn get_bool(&self, key: &str, default: bool) -> Result<bool, ImageIOError> {
        if !&self.options.contains_key(key) {
            return Ok(default);
        }

        match self.options.get(key).map(|v| v.clone()).unwrap().to_lowercase().trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => return Err(ImageIOError::InvalidOptions {
                description: format!("failed to parse option value as a bool: {}", other),
            })
        }
    }

    pub fn get_u32(&self, key: &str, default: u32) -> Result<u32, ImageIOError> {
        if !&self.options.contains_key(key) {
            return Ok(default);
        }

        self.options.get(key).map(|v| v.clone()).unwrap().parse().map_err(|err| ImageIOError::InvalidOptions {
            description: format!("failed to parse option as u32: {}", err),
        })
    }
}
