pub mod utils;

use gluescript::constants;
use gluescript::{GlueNode, RequestBodyType};
use jsonpath_rust::JsonPathFinder;
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::sync::{Arc, Mutex};
use utils::heap;

#[derive(Debug)]
struct RequestError(String);

impl Error for RequestError {}

impl fmt::Display for RequestError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "There is an error: {}", self.0)
	}
}

type MuxNode = Arc<Mutex<GlueNode>>;
type HeapMap = LockedMap<String, String>;
type DepMap = LockedMap<u32, String>;
type LockedMap<K, V> = Arc<Mutex<HashMap<K, V>>>;
type ParallelExecutionLayer = Vec<MuxNode>;
type ExecutionStack = Vec<ParallelExecutionLayer>;

/// A sequential executor of `Runner` instances.
/// 
/// Allow each added `Runner` to share data between subsequent runs.
/// All the data is released on `Stack` end of life.
/// 
/// Can be created empty:
/// ```rust
/// let stack = Stack::new();
/// ```
/// 
/// Or starting from a `GlueNode`:
/// ```rust
/// let stack = Stack::from_root_node(node);
/// ```
pub struct Stack {
	/// The vector of `Runner` instances that will be executed
	/// one by one.
	runners: Vec<Runner>,

	/// A `Stack` holds an HashMap behind an `Arc<Mutex<T>>` to allow
	/// each `GlueNode` executed to access the same memory concurrently
	/// from different tasks
	heap: HeapMap,

	/// An index to the next `Runner` to be executed.
	current: usize,
}

impl Stack {
	/// Create a new `Stack` instance with empty runners `Vec`.
	/// an empty `Arc<Mutex<HashMap>>` is used as heap.
	pub fn new() -> Self {
		Stack {
			runners: vec![],
			heap: heap(),
			current: 0,
		}
	}

	/// Create a new `Stack` instance starting from a `GlueNode`. 
	/// The created `GlueNode` will receive the `Arc` heap map from the fresh `Stack`.
	pub fn from_root_node(root: &GlueNode, log_info: bool) -> Self {
		let mut stack = Stack::new();

		// The same Arc is cloned in every `Runner` of the stack, so all every
		// `GlueNode` will concurrently access to the same memory.
		let runner = Runner::from_root_node(root, Arc::clone(&stack.heap), log_info);

		stack.push_runner(runner);
		stack
	}

	/// Create a new `Stack` instance starting from a `String`.
	/// The supplied `String` is used to create a `Runner`.
	/// 
	/// If the `Runner` creation fails for some reason, an `Err` is returned.
	/// The created `Runner` will receive the `Arc` heap map from the fresh `Stack`.
	pub fn push_runner_from_string(
		self: &mut Self,
		command: &String,
		log_info: bool,
	) -> Result<(), String> {
		// The `Stack` Arc heap is cloned in the `Runner`, so every `GlueNode`
		// contained in it will concurrently access to the same memory.
		let runner = Runner::from_string(command, Arc::clone(&self.heap), log_info)?;

		self.runners.push(runner);
		Ok(())
	}

	/// Add a `Runner` to the `Stack`
	pub fn push_runner(self: &mut Self, mut runner: Runner) -> () {
		// Heap is cloned from the `Stack` so the `Runner` can access
		// the same memory to read and write response variables
		runner.heap = Arc::clone(&self.heap);

		// Runner is simply pushed into the array as owned
		self.runners.push(runner);
	}

	/// Execute the next `Runner` in the `Stack` and increases the 
	pub async fn execute_next(self: &mut Self) -> Result<(), String> {
		// The next runner to be executed is takes by `self.next` index
		let runner = &mut self.runners[self.current];
		
		// next counter is increased before executing the runner as we
		// want to propagate the Err but still be able to continue the
		// execution of the next runner
		self.current += 1;
		
		// runner execution is awaited to be able to use all its results
		// in the subsequent runners.
		runner.execute().await?;
		Ok(())
	}

	/// Get the next `Runner` to be executed wrapped in an `Option`. 
	/// 
	/// Returns None if there is no `Runner` to execute.
	pub fn current(self: &Self) -> Option<&Runner> {
		match self.current {
			x if x > 0 => Some(&self.runners[x]),
			_ => None,
		}
	}

	/// Get the `HeapMap` from the `Stack`
	pub fn heap(self: &Self) -> &HeapMap {
		&self.heap
	}
}

#[derive(Debug)]
pub struct Runner {
	pub layers: ExecutionStack,
	pub depth: usize,
	pub result: Option<String>,
	pub heap: HeapMap,
	pub log_info: bool,
}

impl Runner {
	fn new(heap: HeapMap, log_info: bool) -> Self {
		Runner {
			layers: vec![vec![]],
			depth: 0,
			result: None,
			heap,
			log_info,
		}
	}

	/// Creates a reversed Runner instance containing all
	/// node requests in a root node
	pub fn from_root_node(root: &GlueNode, heap: HeapMap, log_info: bool) -> Self {
		let mut stack = Runner::new(heap, log_info);

		stack.add_node_recursive(&root);
		stack.layers.reverse();

		stack
	}

	/// Creates a reversed Runner instance containing all
	/// node requests in a root node created from string
	pub fn from_string(command: &String, heap: HeapMap, log_info: bool) -> Result<Self, String> {
		match GlueNode::from_string(command) {
			Err(x) => Err(x),
			Ok(x) => Ok(Runner::from_root_node(&x, heap, log_info)),
		}
	}

	pub fn from_file(path: &String, heap: HeapMap, log_info: bool) -> Result<Self, String> {
		let command = match fs::read_to_string(path) {
			Err(x) => return Err(x.to_string()),
			Ok(x) => x,
		};

		match GlueNode::from_string(&command) {
			Err(x) => Err(x),
			Ok(x) => Ok(Runner::from_root_node(&x, heap, log_info)),
		}
	}

	/// Adds a node to the stack automatically determining
	/// the layer it belongs to
	pub fn add_node(self: &mut Self, node: &GlueNode) -> () {
		if self.depth < node.depth {
			self.add_layer();
		}
		self.layers[self.depth].push(Arc::new(Mutex::new(node.to_owned())));
	}

	/// Execute the request stack layer by layer.
	/// Layers are executed one by one, requests inside
	/// layers are executed concurrently using a tokio
	/// task per each request
	pub async fn execute(self: &mut Self) -> Result<(), String> {
		let dependencies_resolutions: Arc<Mutex<HashMap<u32, String>>> =
			Arc::new(Mutex::new(HashMap::new()));

		for layer in self.layers.iter() {
			let mut tasks = vec![];

			for request in layer.iter() {
				let mut req_lock = request.lock().unwrap();
				req_lock.resolve_dependencies(Arc::clone(&dependencies_resolutions))?;

				tasks.push(execute_node(
					Arc::clone(&request),
					Arc::clone(&dependencies_resolutions),
					Arc::clone(&self.heap),
					self.log_info,
				))
			}

			for task in tasks {
				let (depth, response) = match task.await {
					Err(x) => return Err(x),
					Ok(x) => x,
				};

				if depth == 0 {
					self.result = Some(response);
				}
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
	node: MuxNode,
	task_dependencies: DepMap,
	heap: HeapMap,
	log_info: bool,
) -> Result<(usize, String), String> {
	let mut w_node = node.lock().unwrap();

	if w_node.dependencies.len() > 0 {
		let dep_ids: Vec<u32> = w_node.dependencies.iter().map(|dep| dep.id).collect();

		for id in dep_ids.into_iter() {
			w_node.predicate = w_node.predicate.replacen(
				"{}",
				task_dependencies.lock().unwrap().get(&id).unwrap(),
				1,
			);
		}
	}

	match w_node.resolve_predicate() {
		Err(x) => return Err(x),
		_ => (),
	};

	if log_info {
		w_node.print_info();
	}

	let result = match w_node.method.as_str() {
		// we take the result from the heap if it's a saved variable
		constants::REQ => String::from(
			heap.lock()
				.unwrap()
				.get(&w_node.url)
				.expect(constants::ERR_UNRESOLVED_VAR),
		),
		// instead, an http request is fired
		_ => match send_http_request(Arc::clone(&node)).await {
			Err(x) => return Err(x.to_string()),
			Ok(x) => x,
		},
	};

	let is_root = w_node.depth == 0;

	w_node.result = match get_response_value(&w_node.result_selector, &result, !is_root, is_root) {
		Err(x) => return Err(x),
		Ok(x) => x,
	};

	let mut task_dependencies = task_dependencies.lock().unwrap();
	task_dependencies.insert(w_node.id, String::from(&w_node.result));
	drop(task_dependencies);

	if w_node.save_as.is_some() {
		let var_key = String::from(w_node.save_as.clone().unwrap().trim());
		let mut heap = heap.lock().unwrap();
		heap.insert(var_key, String::from(&w_node.result));
		drop(heap);
	}

	Ok((w_node.depth, String::from(&w_node.result)))
}

async fn send_http_request(node: MuxNode) -> Result<String, Box<dyn Error>> {
	let node = node.lock().unwrap();

	let client = match node.method.as_str() {
		constants::GET => Client::new().get(&node.url),
		constants::POST => Client::new().post(&node.url),
		constants::PUT => Client::new().put(&node.url),
		constants::PATCH => Client::new().patch(&node.url),
		constants::DELETE => Client::new().delete(&node.url),
		_ => {
			return Err(Box::new(RequestError(
				constants::ERR_UNKNOWN_METHOD.to_string(),
			)))
		}
	};

	let mut request = match &node.body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		},
	};

	if node.headers.is_some() {
		request = request.headers(node.headers.clone().unwrap());
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
