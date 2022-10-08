use crate::constants;
use lazy_static::lazy_static;
use regex::Regex;

pub fn exclude_quoted_text(input: String) -> String {
	lazy_static! {
		static ref RE: Regex = Regex::new(r#""(.*?)""#).unwrap();
	}

	RE.replace_all(&input, "").to_string()
}

pub fn trim_and_remove_quotes(mut input: String) -> String {
	input = String::from(input.trim());
	if input.starts_with('"') && input.ends_with('"') {
		input = String::from(&input[1..input.len() - 2])
	}
	input
}

/// Return a `(key: String, value: String)` tuple from an attribute
/// string of the form `key=value`
pub fn resolve_key_and_value(attribute: String) -> Result<(String, String), String> {
	// Split key and value as they are divided by `=`
	let mut key_value_array = attribute.trim().split('=');

	// Fail if key is none
	let key = match key_value_array.next() {
		None => return Err(String::from(constants::ERR_UNRESOLVED_ATTR_KEY)),
		Some(x) => x.trim().to_string(),
	};

	// Fail if value is none, removing opening closing quotes if present
	let value = match key_value_array.next() {
		None => return Err(String::from(constants::ERR_UNRESOLVED_ATTR_VAL)),
		Some(x) => trim_and_remove_quotes(x.to_string()),
	};

	Ok((key, value))
}
