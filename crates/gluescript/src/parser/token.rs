use crate::utils::resolve_key_and_value;

use super::{
	errors::{ParserError, ParserErrorType},
	mask::Mask,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	Method(Value),
	Url(Value),
	Selector(Value),
	BodyRaw(Value),
	BodyAttribute(Attribute),
	HeaderAttribute(Attribute),
	SaveAs(Value),
}

impl Token {
	pub fn new(value: Mask, position: usize) -> Result<Self, ParserError> {
		if Token::is_selector(&value.unmasked()) {
			return Ok(Token::Selector(Value::new(
				(&value.unmasked())[1..].to_string(),
			)));
		}

		if Token::is_body_attribute(&value.unmasked()) {
			return Ok(Token::BodyAttribute(Attribute::new(
				(&value.unmasked())[1..].to_string(),
			)));
		}

		if Token::is_headers_attribute(&value.unmasked()) {
			return Ok(Token::HeaderAttribute(Attribute::new(
				(&value.unmasked())[1..].to_string(),
			)));
		}

		if Token::is_save_as(&value.unmasked()) {
			return Ok(Token::SaveAs(Value::new(
				(&value.unmasked())[1..].to_string(),
			)));
		}

		if Token::is_valid_method(&value.unmasked()) && position == 0 {
			return Ok(Token::Method(Value::new(value.unmasked().clone())));
		}

		if position == 1 {
			return Ok(Token::Url(Value::new(value.unmasked().clone())));
		}

		Err(ParserError::new(
			ParserErrorType::InvalidToken,
			format!(
				"Invalid token {} at position {}",
				value.unmasked(),
				position
			),
		))
	}

	fn is_selector(value: &String) -> bool {
		value.starts_with('^')
	}

	fn is_body_attribute(value: &String) -> bool {
		value.starts_with('~')
	}

	fn is_headers_attribute(value: &String) -> bool {
		value.starts_with('*')
	}

	fn is_save_as(value: &String) -> bool {
		value.starts_with('>')
	}

	fn is_valid_method(value: &String) -> bool {
		match value.to_lowercase().as_str() {
			"get" | "post" | "put" | "patch" | "delete" | "req" => true,
			_ => false,
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
	value: String,
}

impl Value {
	pub fn new(value: String) -> Self {
		Value { value }
	}

	pub fn value(self: &Self) -> String {
		self.value.clone()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
	value: String,
}

impl Attribute {
	pub fn new(value: String) -> Self {
		Attribute { value }
	}

	pub fn value(self: &Self) -> Result<(String, String), ParserError> {
		match resolve_key_and_value(self.value.clone()) {
			Err(x) => Err(ParserError::new(
				ParserErrorType::MalformedAttribute,
				x.to_string(),
			)),
			Ok(x) => Ok(x),
		}
	}
}
