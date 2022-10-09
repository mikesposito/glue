use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ParserErrorType {
	InvalidToken,
	MalformedAttribute,
}

#[derive(Debug)]
pub struct ParserError {
	pub error: ParserErrorType,

	pub message: String,
}

impl ParserError {
	pub fn new(error: ParserErrorType, message: String) -> Self {
		ParserError { error, message }
	}
}

impl Error for ParserError {}

impl fmt::Display for ParserError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Parser error: {}", self.message)
	}
}
