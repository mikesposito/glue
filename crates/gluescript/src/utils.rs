use crate::constants;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
	static ref JSON_STRING_RE: Regex = Regex::new(r#"~#-(.*?)-#"#).unwrap();
	static ref SERIALIZATION_PLACEHOLDER_RE: Regex = Regex::new(r#"\|#(\d+)\|"#).unwrap();
}

pub fn exclude_quoted_text(input: String) -> String {
	lazy_static! {
		static ref RE: Regex = Regex::new(r#""(.*?)""#).unwrap();
	}

	RE.replace_all(&input, "").to_string()
}

pub fn extract_and_mask_quoted_text(mut input: String) -> (String, Vec<String>) {
	lazy_static! {
		static ref RE: Regex = Regex::new(r#""([^"\\]|\\.|\\\n)*""#).unwrap();
	}

	let captures: Vec<String> = RE
		.captures_iter(input.as_str())
		.map(|cap| String::from(cap.get(0).unwrap().as_str()))
		.collect();

	for (i, capture) in captures.iter().enumerate() {
		input = input.replacen(capture, &format!(r#""{}""#, i), 1);
	}

	(input, captures)
}

pub fn is_value_a_quoted_reference(input: String) -> bool {
	lazy_static! {
		static ref RE: Regex = Regex::new(r#""(\d+)""#).unwrap();
	}

	RE.is_match(input.as_str())
}

pub fn quoted_reference_to_value(input: String, values: &Vec<String>) -> Result<String, String> {
	lazy_static! {
		static ref RE: Regex = Regex::new(r#"(\d+)"#).unwrap();
	}

	let index = match RE.captures_iter(input.as_str()).nth(0) {
		None => return Err(constants::ERR_UNRESOLVED_VAL.to_string()),
		Some(x) => match x.get(1) {
			None => return Err(constants::ERR_UNRESOLVED_VAL.to_string()),
			Some(x) => x.as_str(),
		},
	};

	Ok(trim_and_remove_quotes(
		values[index.parse::<usize>().unwrap()].clone(),
	))
}

pub fn trim_and_remove_quotes(mut input: String) -> String {
	input = String::from(input.trim());
	if input.starts_with('"') && input.ends_with('"') {
		input = String::from(&input[1..input.len() - 1])
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
		Some(x) => x.trim().to_string(),
	};

	Ok((key, value))
}

pub fn get_raw_json_body(input: &String) -> Option<String> {
	match JSON_STRING_RE.captures_iter(input.as_str()).nth(0) {
		None => None,
		Some(x) => match x.get(1) {
			None => None,
			Some(x) => Some(x.as_str().to_string()),
		},
	}
}

pub fn serialize(input: String) -> (String, Vec<String>) {
	let mut output = input.clone();

	let captures: Vec<String> = JSON_STRING_RE
		.captures_iter(input.as_str())
		.map(|cap| String::from(cap.get(0).unwrap().as_str()))
		.collect();

	(0..captures.len()).for_each(|i| {
		output = JSON_STRING_RE
			.replacen(output.as_str(), 1, format!("|#{i}|"))
			.to_string();
	});

	(output, captures)
}

pub fn deserialize(input: String, components: &Vec<String>) -> String {
	let mut output = input.clone();

	components.iter().enumerate().for_each(|(i, component)| {
		output = output.replacen(&format!("|#{i}|"), component.as_str(), 1);
	});

	output
}

pub fn remove_serialization_placeholders(input: &String) -> String {
	SERIALIZATION_PLACEHOLDER_RE
		.replace_all(input.as_str(), "")
		.to_string()
}
