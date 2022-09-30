pub mod constants;

use colored::*;
use rand::prelude::random;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;

const OPEN_DELIMITER: char = '{';
const CLOSE_DELIMITER: char = '}';

#[derive(Debug, Clone)]
pub struct GlueNode {
	pub id: u32,
	pub command: String,
	pub predicate: String,
	pub method: String,
	pub url: String,
	pub result_selector: String,
	pub headers: Option<HeaderMap>,
	pub body: Option<RequestBody>,
	pub dependencies: Vec<GlueNode>,
	pub depth: usize,
	pub result: String,
}

#[derive(Debug, Clone)]
pub struct RequestBody {
	pub body_type: RequestBodyType,
	pub value: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum RequestBodyType {
	JSON,
	FORM,
}

impl GlueNode {
	pub fn new(command: &String, depth: usize) -> Self {
		GlueNode {
			id: random(),
			command: String::from(command),
			predicate: String::from(""),
			method: String::from(""),
			url: String::from(""),
			headers: None,
			body: None,
			result_selector: String::from(""),
			dependencies: Vec::new(),
			depth,
			result: String::from(""),
		}
	}

	/// Takes a String as input and returns a GlueNode
	/// instance wrapped in Result. Returns an Err Result if
	/// something goes wrong with the parsing
	pub fn from_string(command: &String) -> Result<Self, String> {
		let mut root_node = GlueNode::new(command, 0);

		match root_node.build_tree_recursive() {
			Err(x) => Err(x),
			Ok(_) => Ok(root_node),
		}
	}

	/// Recursively parses the command up until a closing
	/// delimiter is encountered. It calls itself when a new
	/// open delimiter is encountered. Returns a wrapped usize
	/// to the function caller to allow it to determine at which
	/// point of the command the dependency was closed
	fn build_tree_recursive(self: &mut Self) -> Result<usize, String> {
		let mut close_dep_at: usize = 0;
		let mut skip_till: usize = 0;

		for (i, char) in &mut self.command.chars().enumerate() {
			// skip parsing if looping through dependency
			if skip_till > 0 && i <= skip_till + 1 {
				continue;
			}

			match char {
				OPEN_DELIMITER => {
					let mut dependency =
						GlueNode::new(&self.command[(i + 1)..].to_string(), self.depth + 1);

					match dependency.build_tree_recursive() {
						Err(x) => return Err(x),
						Ok(x) => skip_till = x + i,
					};

					self.dependencies.push(dependency);
					self.predicate.push_str("{}");
				}

				CLOSE_DELIMITER => {
					close_dep_at = i;
					// ignore everything after the closing delimiter
					break;
				}

				_ => self.predicate.push(char),
			}
		}

		Ok(close_dep_at)
	}

	/// Reads the predicate and tries to resolve
	/// method, url and selector
	pub fn resolve_predicate(self: &mut Self) -> Result<(), String> {
		self.resolve_method()?;
		self.resolve_url()?;
		self.resolve_selector();
		self.resolve_headers()?;
		self.resolve_body()?;

		Ok(())
	}

	fn resolve_method(self: &mut Self) -> Result<(), String> {
		self.method = match self.predicate.trim().split(' ').nth(0) {
			None => return Err(String::from(constants::ERR_UNRESOLVED_METHOD)),
			Some(x) => x.to_string().replace("\n", ""),
		};

		Ok(())
	}

	fn resolve_url(self: &mut Self) -> Result<(), String> {
		let resource = match self.predicate.trim().split(' ').nth(1) {
			None => return Err(String::from(constants::ERR_UNRESOLVED_URL)),
			Some(x) => x,
		};

		self.url = match resource.split(['^', '~', '*']).nth(0) {
			None => return Err(String::from(constants::ERR_UNRESOLVED_URL)),
			Some(x) => x.to_string().replace("\n", ""),
		};

		Ok(())
	}

	fn resolve_selector(self: &mut Self) -> () {
		self.result_selector = match self.predicate.split('^').nth(1) {
			None => "".to_string(),
			Some(x) => x.to_string(),
		};
	}

	fn resolve_headers(self: &mut Self) -> Result<(), String> {
		let mut request_headers = HeaderMap::new();
		let mut headers_parts = self.predicate.split('*');
		headers_parts.next(); // The first is always the url and selector

		for attribute in headers_parts.into_iter() {
			let sanitized = attribute.split(['\n', '\t', ' ']).nth(0).unwrap();
			let mut key_value_array = sanitized.trim().split('=');

			let key = match key_value_array.next() {
				None => return Err(String::from(constants::ERR_UNRESOLVED_ATTR_KEY)),
				Some(x) => x.trim().to_string(),
			};

			let value = match key_value_array.next() {
				None => return Err(String::from(constants::ERR_UNRESOLVED_ATTR_VAL)),
				Some(x) => x.trim().to_string(),
			};

			let header_name = match HeaderName::from_lowercase(key.to_lowercase().as_bytes()) {
				Err(x) => return Err(x.to_string()),
				Ok(x) => x,
			};

			let header_value = match HeaderValue::from_str(&value[..]) {
				Err(x) => return Err(x.to_string()),
				Ok(x) => x,
			};

			request_headers.insert(header_name, header_value);
		}

		if !request_headers.is_empty() {
			self.headers = Some(request_headers);
		}

		Ok(())
	}

	fn resolve_body(self: &mut Self) -> Result<(), String> {
		let mut request_body: HashMap<String, String> = HashMap::new();
		let mut body_parts = self.predicate.split('~');
		body_parts.next(); // The first is always the url and selector

		for attribute in body_parts.into_iter() {
			let sanitized = attribute.split(['\n', '\t', ' ']).nth(0).unwrap();
			let mut key_value_array = sanitized.trim().split('=');

			let key = match key_value_array.next() {
				None => return Err(String::from(constants::ERR_UNRESOLVED_ATTR_KEY)),
				Some(x) => x.trim().to_string(),
			};

			let value = match key_value_array.next() {
				None => return Err(String::from(constants::ERR_UNRESOLVED_ATTR_VAL)),
				Some(x) => x.trim().to_string(),
			};

			request_body.insert(key, value);
		}

		if !request_body.is_empty() {
			self.body = Some(RequestBody::new(RequestBodyType::JSON, request_body));
		}

		Ok(())
	}

	pub fn print_info(self: &Self) -> () {
		println!(
			"> {} {}",
			self.method.to_uppercase().truecolor(110, 110, 110),
			self.url.truecolor(110, 110, 110)
		);

		match &self.body {
			Some(x) => {
				for (key, value) in &x.value {
					println!(
						"\t{}{}{}",
						key.truecolor(110, 110, 110),
						"=".truecolor(110, 110, 110),
						value.truecolor(110, 110, 110)
					)
				}
			}
			_ => (),
		}
	}
}

impl RequestBody {
	pub fn new(body_type: RequestBodyType, value: HashMap<String, String>) -> Self {
		RequestBody { body_type, value }
	}
}
