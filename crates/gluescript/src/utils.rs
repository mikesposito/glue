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
