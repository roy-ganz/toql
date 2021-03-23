
use std::fmt;

#[derive(Debug)]
pub enum DeserializeError {
    SelectionExpected(String),
    StreamEnd,
    ConversionFailed(String, String)
}


impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeserializeError::SelectionExpected(ref field ) => write!(f, "expected field `{}` to be selected, but got unselected.", field),
            DeserializeError::StreamEnd => write!(f, "expected stream to contain more selections"),
            DeserializeError::ConversionFailed( ref field,  ref cause) => write!(f, "unable to convert field `{}`, because `{}`", field, cause),
        }
    }
}