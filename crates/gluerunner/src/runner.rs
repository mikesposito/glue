use crate::MuxNode;

use super::{http::execute_node, ExecutionStack, GlueNode, HeapMap};
use std::{
	fs,
	sync::{Arc, Mutex},
};

/// A component responsible for `GlueNode` execution.
/// All `GlueNode` are sorted into different layers based on their depth and
/// dependance between each other.
///
/// Layers are executed one-at-a-time, and every `GlueNode` of the same layer will run
/// on parallel.
#[derive(Debug)]
pub struct Runner {
	/// Root `GlueNode` of the runner. Holds the ownership of the main request 
	/// struct to be executed. `self.layers` are build starting from this. 
	pub root: MuxNode,

	/// Vector of Vector of `Runner`. Each `Runner` is behind an `Arc<Mutex<T>>`
	/// to enable concurrent access to the same shared memory between runners.
	/// Every `Runner` is a reference to a `MuxNode` part of `Runner.root`, so
	/// reads and writes on this object are reflected on `Runner.root`.
	pub layers: ExecutionStack,

	/// Number representing `Runner` max depth, equal to the max depth between
	/// all the `GlueNode` in it.
	pub depth: usize,

	/// Option containing the final result of the `Runner`, also equal to the result
	/// of the `GlueNode` with depth 0
	pub result: Option<String>,

	/// A shared memory map used to read and write `GlueNode` results their reuse in
	/// subsequent runs.
	pub heap: HeapMap,

	/// Boolean to toggle verbose mode.
	/// In verbose mode each executed `GlueNode` also prints request info
	/// before is run.
	pub log_info: bool,
}

impl Runner {
	/// A new `Runner` consists in an empty `HeapMap` and a single, empty layer.
	/// For convenience, `depth` will default to 0 even if there's no `GlueNode`
	/// with `depth` 0 in it.
	fn new(root: MuxNode, heap: HeapMap, log_info: bool) -> Self {
		// Initial `Runner` instance with `root` ownership and initial empty
		// layers and heap.
		let mut runner = Runner {
			root,
			layers: vec![vec![]],
			depth: 0,
			result: None,
			heap,
			log_info,
		};

		// `runner` layers are built starting from `root`, creating multiple
		// references behind mutexes and placing them in the layers matrix.
		runner.build_layers();
		runner
	}

	/// Create a Runner instance containing the provided `GlueNode` along with
	/// all its dependency nodes.
	pub fn from_root_node(root: GlueNode, heap: HeapMap, log_info: bool) -> Self {
		// The runner is created using the provided heap map and wrapping
		// `root` in a Mutex, as will be mutually accessed.
		Runner::new(Arc::new(Mutex::new(root)), heap, log_info)
	}

	/// Create a `GlueNode` instance starting from provided `command` and then
	/// create a Runner instance from the `GlueNode`, along with all its dependency nodes.
	pub fn from_string(command: &String, heap: HeapMap, log_info: bool) -> Result<Self, String> {
		match GlueNode::from_string(command) {
			Err(x) => Err(x),
			Ok(x) => Ok(Runner::from_root_node(x, heap, log_info)),
		}
	}

	/// Create a `GlueNode` instance starting from file content at `path` and then
	/// create a Runner instance from the `GlueNode`, along with all its dependency nodes.
	pub fn from_file(path: &String, heap: HeapMap, log_info: bool) -> Result<Self, String> {
		// Get file at `path` or propagate error.
		let command = match fs::read_to_string(path) {
			Err(x) => return Err(x.to_string()),
			Ok(x) => x,
		};

		// Create `GlueNode` from file content, propagate error on fail.
		match GlueNode::from_string(&command) {
			Err(x) => Err(x),
			Ok(x) => Ok(Runner::from_root_node(x, heap, log_info)),
		}
	}

	/// Add a `GlueNode` to the `Runner` automatically determining the layer it belongs to.
	pub fn add_node(self: &mut Self, node: MuxNode) -> () {
		// Acquire readable lock on `node` mutex
		let r_node = node.lock().unwrap();

		if self.depth < r_node.depth {
			// If layer is not existing yet, create it.
			self.add_layer();
		}

		// Release lock on node mutex
		drop(r_node);

		// Node is pushed wrapped in a `Arc<Mutex<T>>` to be shared across threads and
		// accessed concurrently.
		self.layers[self.depth].push(node);
	}

	/// Execute the `Runner` layer by layer.
	/// Layers are executed one-at-a-time, while every `GlueNode`
	/// of the same layer is executed concurrently.
	///
	/// A green tokio task is spawned for each `GlueNode` to be executed and all tasks
	/// are awaited before continuing to the next layer
	pub async fn execute(self: &mut Self) -> Result<(), String> {
		// Each layer is executed granularly, and waiting for its termination
		// before continuing to the next layer.
		for layer in self.layers.iter() {
			// An array of lazy tasks to be awaited.
			let mut tasks = vec![];

			for request in layer.iter() {
				// We lock the dependency map in read mode to let the `GlueNode`
				// resolve its own dependencies based on previous runs.
				let mut w_request = request.lock().unwrap();

				// As all dependencies should have been executed already, `request`
				// can resolve all of its dependencies.
				w_request.resolve_dependencies();

				// Execution routine is pushed into execution pipe, ready to be polled
				tasks.push(execute_node(
					Arc::clone(&request),
					Arc::clone(&self.heap),
					self.log_info,
				))
			}

			for task in tasks {
				// Polling on each task to block execution flow till they are ready.
				let (depth, response) = match task.await {
					Err(x) => return Err(x),
					Ok(x) => x,
				};

				if depth == 0 {
					// If the depth of the `GlueNode` is 0, it means it is the root one,
					// and last to be executed, so its result also equals to the `Runner` result.
					self.result = Some(response);
				}
			}
		}

		Ok(())
	}

	/// Create a new empty layer in the `Runner`, increasing also
	/// its depth.
	fn add_layer(self: &mut Self) -> () {
		self.layers.push(vec![]);
		self.depth += 1;
	}

	/// Recursively add `GlueNode` to stack, along with all its dependencies.
	fn add_node_recursive(self: &mut Self, node: MuxNode) -> () {
		self.add_node(Arc::clone(&node));

		// Acquire readable lock on `node` to iterate dependencies
		let r_node = node.lock().unwrap();

		for dep_node in &r_node.dependencies {
			// Recursively add a Mutex clone of the dependency
			// to the layers matrix.
			self.add_node_recursive(Arc::clone(dep_node));
		}
	}

	/// Populate `self.layers` with dependency referenced mutexes
	/// starting from `self.root`
	fn build_layers(self: &mut Self) -> () {
		// Layers are built starting from a new Mutex clone
		self.add_node_recursive(Arc::clone(&self.root));

		// Layers array is reversed to be able to execute the `GlueNode` with
		// highest depth first, and incrementally going to the 0 one.
		// This is necessary as depth with lower depth will always be dependant
		// on the ones with higher depth.
		self.layers.reverse();
	}
}
