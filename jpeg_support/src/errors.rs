use custom_error::custom_error;

custom_error! {pub JPEGReaderError
    InvalidHeader {description: String} = "Invalid header: {description}",
    InvalidSegment {description: String} = "Invalid segment: {description}",
    InvalidEncodedData {description: String} = "Invalid encoded data: {description}",
}