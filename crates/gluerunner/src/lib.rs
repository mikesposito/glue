use gluescript::constants;
use gluescript::{GlueNode, RequestBody, RequestBodyType};
use jsonpath_rust::JsonPathFinder;
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;

#[derive(Debug)]
struct RequestError(String);

impl Error for RequestError {}

impl fmt::Display for RequestError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "There is an error: {}", self.0)
	}
}

#[derive(Debug)]
pub struct Runner {
	pub layers: Vec<Vec<GlueNode>>,
	pub depth: usize,
	pub result: Option<String>,
	pub heap: HashMap<String, String>,
	pub log_info: bool,
}

impl Runner {
	fn new(log_info: bool) -> Self {
		Runner {
			layers: vec![vec![]],
			depth: 0,
			result: None,
			heap: HashMap::new(),
			log_info,
		}
	}

	/// Creates a reversed Runner instance containing all
	/// node requests in a root node
	pub fn from_root_node(root: &GlueNode, log_info: bool) -> Self {
		let mut stack = Runner::new(log_info);

		stack.add_node_recursive(&root);
		stack.reverse();

		stack
	}

	/// Creates a reversed Runner instance containing all
	/// node requests in a root node created from string
	pub fn from_string(command: &String, log_info: bool) -> Result<Self, String> {
		match GlueNode::from_string(command) {
			Err(x) => Err(x),
			Ok(x) => Ok(Runner::from_root_node(&x, log_info)),
		}
	}

	pub fn from_file(path: &String, log_info: bool) -> Result<Self, String> {
		let command = match fs::read_to_string(path) {
			Err(x) => return Err(x.to_string()),
			Ok(x) => x,
		};

		match GlueNode::from_string(&command) {
			Err(x) => Err(x),
			Ok(x) => Ok(Runner::from_root_node(&x, log_info)),
		}
	}

	/// Adds a node to the stack automatically determining
	/// the layer it belongs to
	pub fn add_node(self: &mut Self, node: &GlueNode) -> () {
		if self.depth < node.depth {
			self.add_layer();
		}
		self.layers[self.depth].push(node.to_owned());
	}

	pub fn reverse(self: &mut Self) -> () {
		self.layers.reverse();
	}

	pub async fn execute(self: &mut Self) -> Result<(), String> {
		let mut dependencies_resolutions: HashMap<u32, String> = HashMap::new();

		for layer in &mut self.layers {
			let mut tasks = vec![];

			for (i, request) in layer.into_iter().enumerate() {
				let task_dependencies = dependencies_resolutions.clone();
				let node = request.clone();
				tasks.push((
					i,
					tokio::spawn(execute_node(node, task_dependencies, self.log_info)),
				));
			}

			for (i, task) in tasks {
				let result = match task.await {
					Err(x) => return Err(x.to_string()),
					Ok(x) => x,
				};

				let executed_node = match result {
					Err(x) => return Err(x),
					Ok(x) => x,
				};

				if executed_node.depth == 0 {
					self.result = Some(String::from(&executed_node.result));
				}

				if executed_node.save_as.is_some() {
					let var_key = String::from(executed_node.clone().save_as.unwrap().trim());
					self.heap
						.insert(var_key, String::from(&executed_node.result));
				}

				dependencies_resolutions
					.insert(executed_node.id, String::from(&executed_node.result));
				layer[i] = executed_node;
			}
		}

		Ok(())
	}

	fn add_layer(self: &mut Self) -> () {
		self.layers.push(vec![]);
		self.depth += 1;
	}

	fn add_node_recursive(self: &mut Self, node: &GlueNode) -> () {
		self.add_node(node);
		for dep_node in &node.dependencies {
			self.add_node_recursive(dep_node);
		}
	}
}

async fn execute_node(
	mut node: GlueNode,
	task_dependencies: HashMap<u32, String>,
	log_info: bool,
) -> Result<GlueNode, String> {
	if node.dependencies.len() > 0 {
		for dependency in &node.dependencies {
			let dependency_result = task_dependencies.get(&dependency.id).unwrap();
			node.predicate = node.predicate.replacen("{}", dependency_result, 1);
		}
	}

	match node.resolve_predicate() {
		Err(x) => return Err(x),
		_ => (),
	};

	if log_info {
		node.print_info();
	}

	let result = match http_request(&node.method, &node.url, &node.headers, &node.body).await {
		Err(x) => return Err(x.to_string()),
		Ok(x) => x,
	};

	let is_root = node.depth == 0;

	node.result = match get_response_value(&node.result_selector, &result, !is_root, is_root) {
		Err(x) => return Err(x),
		Ok(x) => x,
	};

	Ok(node)
}

async fn http_request(
	method: &String,
	url: &String,
	headers: &Option<HeaderMap>,
	body: &Option<RequestBody>,
) -> Result<String, Box<dyn Error>> {
	let client = match method.as_str() {
		constants::GET => Client::new().get(url),
		constants::POST => Client::new().post(url),
		constants::PUT => Client::new().put(url),
		constants::PATCH => Client::new().patch(url),
		constants::DELETE => Client::new().delete(url),
		_ => {
			return Err(Box::new(RequestError(
				constants::ERR_UNKNOWN_METHOD.to_string(),
			)))
		}
	};

	let mut request = match body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		},
	};

	if headers.is_some() {
		request = request.headers(headers.clone().unwrap());
	}

	let response = request.send().await?.text().await?;
	Ok(response)
}

fn get_response_value(
	path: &String,
	response: &String,
	just_first_slice_value: bool,
	pretty: bool,
) -> Result<String, String> {
	// No path provided so we return response
	// as is
	if path.len() == 0 {
		return Ok(String::from(response));
	}

	// Path has been provided so we suppose to have
	// a json response
	let json_selectable = match JsonPathFinder::from_str(&response[..], &path[..]) {
		Err(x) => return Err(x),
		Ok(x) => x,
	};

	// JSONPath returns an array with results as default
	// but sometimes what we need from response
	// is just a single string value
	if just_first_slice_value {
		return match json_selectable.find_slice().to_vec()[0].as_str() {
			None => Err(format!("Could not select value to use with this selector: \n{path} \non this response: \n{response}")),
			Some(x) => Ok(String::from(x)),
		};
	}

	// return prettified result string
	if pretty {
		return match serde_json::to_string_pretty(json_selectable.find_slice().as_slice()) {
			Err(x) => Err(x.to_string()),
			Ok(x) => Ok(x),
		};
	}

	// ..or non-prettified
	match serde_json::to_string(json_selectable.find_slice().as_slice()) {
		Err(x) => Err(x.to_string()),
		Ok(x) => Ok(x),
	}
}
