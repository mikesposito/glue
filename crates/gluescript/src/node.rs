use crate::{
	constants, exclude_quoted_text,
	utils::{
		extract_and_mask_quoted_text, get_raw_json_body, is_value_a_quoted_reference,
		quoted_reference_to_value, remove_serialization_placeholders, resolve_key_and_value,
	},
	ParserError, ParserErrorType, RequestBody, RequestBodyType, Serialized,
};
use colored::*;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

/// Main structure for holding request data.
///
/// Its components are calculated starting from its `command`,
/// over which an iterative parsing process is done to resolve all other
/// parts.
///
/// Each `GlueNode` instance may contain other instances underneath as
/// dependencies that have to be resolved and executed before the main one
/// is resolved and executed.
#[derive(Debug, Clone)]
pub struct GlueNode {
	/// `Serialized` value associated at creation time and used
	/// to calculate other struct data after its parsing.
	pub command: Serialized,

	/// String value associated after `command` parsing.
	/// Similar use of `command`, but it is intended to
	/// not include dependencies (a `{}` placeholder is located
	/// in change of them).
	pub predicate: String,

	/// Request HTTP method.
	pub method: String,

	/// Request canonical URL.
	pub url: String,

	/// JSONPath selector used to select a value from a JSON
	/// response after its end.
	pub result_selector: String,

	/// HashMap containing custom headers to attach to request.
	/// If `None`, only default headers will be used.
	pub headers: Option<HeaderMap>,

	/// HashMap containing body to attach to request.
	/// If `None`, request will have an empty body.
	pub body: Option<RequestBody>,

	/// Collection of child `GlueNode` needed as dependencies
	/// for the request to be resolved.
	pub dependencies: Vec<Arc<Mutex<GlueNode>>>,

	/// Depth of the `GlueNode` in the dep tree:
	/// 0 if root node, 1 if first dependency in the graph,
	/// 2 if dependency of another dependency etc..
	pub depth: usize,

	/// Result of `GlueNode` http execution and response parsing.
	pub result: String,

	/// Key to be used to save the `GlueNode` result.
	/// Response will be ephemeral if `None` is provided.
	pub save_as: Option<String>,
}

impl GlueNode {
	/// Create a new `GlueNode` instance from a `command` literal
	/// and a `depth`, indicating its position in a `GlueNode` tree
	/// structure.
	pub fn new(command: &String, depth: usize) -> Self {
		GlueNode {
			command: Serialized::new(command.to_owned()),
			predicate: String::from(""),
			method: String::from(""),
			url: String::from(""),
			headers: None,
			body: None,
			result_selector: String::from(""),
			dependencies: Vec::new(),
			depth,
			result: String::from(""),
			save_as: None,
		}
	}

	/// Create a `GlueNode` instance starting from a `command`.
	/// Returns an `Err` if something goes wrong with the parsing.
	pub fn from_string(command: &String) -> Result<Self, String> {
		let mut root_node = GlueNode::new(command, 0);

		// Parse the `root_node` command to build its predicate.
		// Propagate Err on fail.
		root_node.build_tree_recursive()?;
		Ok(root_node)
	}

	/// Recursively parse `self.command` up until a closing
	/// delimiter is encountered.
	///
	/// Call itself when a new open delimiter is encountered.
	///
	/// Return a wrapped usize to the function caller to allow it to
	/// determine at which point of the `self.command` the dependency was closed.
	fn build_tree_recursive(self: &mut Self) -> Result<usize, String> {
		// This holds an index to the next dependency closing delimiter while
		// iterating over chars of `self.command`.
		// This index is used to skip parsing if iterating in a `command` substr
		// that is part of a dependency, because its parsing will be already done
		// by another iteration of `build_tree_recursive()`
		let mut skip_till: usize = 0;

		// Parse the command, char by char.
		for (i, char) in &mut self.command.serialized().chars().enumerate() {
			// Skip parsing if cursor is in a dependency
			if skip_till > 0 && i <= skip_till + 1 {
				continue;
			}

			match char {
				// If an open delimiter is hit the control flow must be passed
				// to another iteration of the function, so the predicate
				// can be parsed for the dependency.
				constants::OPEN_DELIMITER => {
					// Create a new `GlueNode` instance from a `command`
					// based on a substring of `self.command`, starting from
					// the current cursor index. Depth is incrementally assigned.
					let mut dependency = GlueNode::new(
						&self.command.serialized()[(i + 1)..].to_string(),
						self.depth + 1,
					);

					// build_tree_recursive() is called for the newly created dependency
					// and its result is used to know where is the next closing delimiter.
					// Err is propagated on dependency parsing failure.
					skip_till = dependency.build_tree_recursive()? + i;

					// Dependency is pushed into the `dependencies` collection.
					self.dependencies.push(Arc::new(Mutex::new(dependency)));

					// A `{}` is added to `self.predicate` so it will be possible
					// to replace it afterwards with the actual dependency
					// result.
					self.predicate.push_str("{}");
				}

				// If a closing delimiter is hit, the function is terminated
				// and the current cursor index is returned to the caller.
				constants::CLOSE_DELIMITER => {
					return Ok(i);
				}

				// In any other case, the char is added to `self.predicate`.
				_ => self.predicate.push(char),
			}
		}

		// If this statement is reached, then the `GlueNode`
		// that is executing this function is not a dependency
		// of another node, so 0 is returned.
		Ok(0)
	}

	/// Parse the predicate and tries to resolve other parts of `GlueNode`.
	/// Return Err on any resolve failure.
	pub fn resolve_predicate(self: &mut Self) -> Result<(), String> {
		self.resolve_method()?;
		self.resolve_url()?;
		self.resolve_selector();
		self.resolve_save_as();
		self.resolve_headers()?;
		self.resolve_body()?;

		Ok(())
	}

	/// Resolve http request method from `self.predicate`.
	/// Error is returned is no method is found.
	fn resolve_method(self: &mut Self) -> Result<(), String> {
		// Method must always be the first part of the predicate, followed
		// by a white space.
		self.method = match self.predicate.trim().split(' ').nth(0) {
			None => return Err(String::from(constants::ERR_UNRESOLVED_METHOD)),
			Some(x) => x.to_string().replace("\n", ""),
		};

		Ok(())
	}

	/// Resolve http request canonical url from `self.predicate`.
	/// Error is returned is no url is found.
	fn resolve_url(self: &mut Self) -> Result<(), String> {
		let clean_predicate = remove_serialization_placeholders(&self.predicate);
		// Url should always be the second token of the predicate,
		// preceded by a space, but not necessarily followed by it.
		let resource = match clean_predicate.trim().split(' ').nth(1) {
			None => return Err(String::from(constants::ERR_UNRESOLVED_URL)),
			Some(x) => x,
		};

		// Exclude every other operator from the url
		self.url = match exclude_quoted_text(resource.to_string())
			.split(['^', '~', '*'])
			.nth(0)
		{
			None => return Err(String::from(constants::ERR_UNRESOLVED_URL)),
			Some(x) => x.to_string().replace("\n", ""),
		};

		Ok(())
	}

	/// Resolve request response JSONPath selector from predicate.
	/// Empty string is returned if no selector is found.
	fn resolve_selector(self: &mut Self) -> () {
		self.result_selector = match exclude_quoted_text(String::from(&self.predicate))
			.split('^')
			.nth(1)
		{
			None => "".to_string(),
			Some(x) => x.to_string(),
		};
	}

	/// Set header attribute to `GlueNode`.
	/// Propagate `ParserError` on invalid header.
	pub fn set_header(self: &mut Self, header: (String, String)) -> Result<(), ParserError> {
		let (key, val) = header;

		// Create header name from lowercase of `key`
		let header_name = match HeaderName::from_lowercase(key.to_lowercase().as_bytes()) {
			Err(x) => {
				return Err(ParserError::new(
					ParserErrorType::MalformedAttribute,
					x.to_string(),
				))
			}
			Ok(x) => x,
		};

		// Create header name from lowercase of `value`, removing opening
		// and closing quotes if present
		let header_value = match HeaderValue::from_str(&val[..]) {
			Err(x) => {
				return Err(ParserError::new(
					ParserErrorType::MalformedAttribute,
					x.to_string(),
				))
			}
			Ok(x) => x,
		};

		if let Some(map) = &mut self.headers {
			// Insert header in existing header map
			map.insert(header_name, header_value);
		} else {
			// Create fresh map.
			let mut map = HeaderMap::new();

			// Insert header in new map.
			map.insert(header_name, header_value);

			// Update node headers.
			self.headers = Some(map);
		}

		Ok(())
	}

	/// Set body attribute to `GlueNode`.
	/// Reset raw body if any.
	pub fn set_body_attribute(self: &mut Self, attribute: (String, String)) -> () {
		let (key, val) = attribute;

		if let Some(body) = &mut self.body {
			// Add key-value pair to body map
			body.value.insert(key, val);

			// Set body type to JSON
			body.body_type = RequestBodyType::JSON;
		} else {
			// Create new map with attribute.
			let mut map = HashMap::new();
			map.insert(key, val);

			// Build request body
			let body = Some(RequestBody::new(RequestBodyType::JSON, Some(map), None));

			// Update node body.
			self.body = body;
		}
	}

	/// Set raw value to body.
	/// Reset attributes if any.
	pub fn set_body_raw(self: &mut Self, raw_body: String) -> () {
		self.body = Some(RequestBody::new(
			RequestBodyType::ARBITRARY,
			None,
			Some(raw_body),
		));
	}

	/// Resolve http request headers `self.predicate`.
	/// Err is returned on failure.
	fn resolve_headers(self: &mut Self) -> Result<(), String> {
		let mut request_headers = HeaderMap::new();

		// Get a sanitized string that excludes text between quotes.
		// Also save the extracted text in a vector to later reuse it.
		let (sanitized, quoted_text) = extract_and_mask_quoted_text(self.predicate.clone());

		// Divide the headers in parts, as each header is always
		// preceded by `*`.
		let mut headers_parts = sanitized.split('*');

		// The first is always the url and selector
		headers_parts.next();

		for attribute in headers_parts.into_iter() {
			// Sanitize the attribute removing any other operator from it
			let sanitized = attribute.split(['\n', '\t', '^', '~']).nth(0).unwrap();

			// Extract key and value from attribute
			let (key, mut value) = resolve_key_and_value(sanitized.to_string())?;

			if is_value_a_quoted_reference(value.clone()) {
				value = quoted_reference_to_value(value, &quoted_text)?;
			}

			// Create header name from lowercase of `key`
			let header_name = match HeaderName::from_lowercase(key.to_lowercase().as_bytes()) {
				Err(x) => return Err(x.to_string()),
				Ok(x) => x,
			};

			// Create header name from lowercase of `value`, removing opening
			// and closing quotes if present
			let header_value = match HeaderValue::from_str(&value[..]) {
				Err(x) => return Err(x.to_string()),
				Ok(x) => x,
			};

			// Add key-value pair to Headers map
			request_headers.insert(header_name, header_value);
		}

		// Set `GlueNode` headers map if at least one attribute has been
		// parsed.
		if !request_headers.is_empty() {
			self.headers = Some(request_headers);
		}

		Ok(())
	}

	/// Resolve http request body `self.predicate`.
	/// Err is returned on failure.
	fn resolve_body(self: &mut Self) -> Result<(), String> {
		// Predicate is deserialized using command serialization components
		self.predicate = self.command.deserialize_part(self.predicate.clone());

		match get_raw_json_body(&self.predicate) {
			// If there is no raw json in the predicate, start looking
			// at single attributes.
			None => {
				let mut request_body: HashMap<String, String> = HashMap::new();

				// Get a sanitized string that excludes text between quotes.
				// Also save the extracted text in a vector to later reuse it.
				let (sanitized, quoted_text) = extract_and_mask_quoted_text(self.predicate.clone());

				// Divide the body attributes in parts, as each header is always
				// preceded by `~`.
				let mut body_parts = sanitized.split('~');

				// The first is always the url and selector
				body_parts.next();

				for attribute in body_parts.into_iter() {
					// Sanitize the attribute removing any other operator from it
					let sanitized = attribute.split(['\n', '\t', '^', '~']).nth(0).unwrap();

					// Extract key and value from attribute
					let (key, mut value) = resolve_key_and_value(sanitized.to_string())?;

					// If value is a quoted reference (value temporary removed because
					// was quoted) - then replace reference with real value
					if is_value_a_quoted_reference(value.clone()) {
						value = quoted_reference_to_value(value, &quoted_text)?;
					}

					// Add key-value pair to body map
					request_body.insert(key, value);
				}

				// Set `GlueNode` body map if at least one attribute has been parsed.
				if !request_body.is_empty() {
					self.body = Some(RequestBody::new(
						RequestBodyType::JSON,
						Some(request_body),
						None,
					));
				}
			}

			// Append raw json to body in case it has been found.
			Some(json) => {
				self.body = Some(RequestBody::new(
					RequestBodyType::ARBITRARY,
					None,
					Some(json),
				));
			}
		}

		Ok(())
	}

	/// Resolve `self.save_as` starting from predicate, excluding all the
	/// text between quotes.
	fn resolve_save_as(self: &mut Self) -> () {
		self.save_as = match exclude_quoted_text(String::from(&self.predicate))
			.split('>')
			.nth(1)
		{
			None => None,
			Some(x) => Some(x.to_string()),
		};
	}

	/// Replace all `{}` placeholders from predicate with dependencies
	/// results taken from a shared memory.
	pub fn resolve_dependencies(self: &mut Self) -> () {
		if self.dependencies.len() > 0 {
			// Dependencies are always in the same order of `{}` placeholders
			for dependency in &self.dependencies {
				// Acquire read lock on the dependency mutex
				let dependency = dependency.lock().unwrap();

				// Replace the next placeholder `{}` in the predicate with the actual
				// dependency value.
				self.predicate = self.predicate.replacen("{}", &dependency.result, 1);
			}
		}
	}

	/// Print colored `GlueNode` info
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
