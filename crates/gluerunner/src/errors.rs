use std::{error::Error, fmt};

#[derive(Debug)]
pub struct RequestError(pub String);

impl Error for RequestError {}

impl fmt::Display for RequestError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "There is an error: {}", self.0)
	}
}
