use crate::{heap, HeapMap, Runner};
use gluescript::GlueNode;
use std::sync::Arc;

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
			x if x > 0 => Some(&self.runners[x - 1]),
			_ => None,
		}
	}

	/// Get the `HeapMap` from the `Stack`
	pub fn heap(self: &Self) -> &HeapMap {
		&self.heap
	}
}
