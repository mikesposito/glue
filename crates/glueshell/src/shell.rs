use crate::utils::{print_err, prompt};
use gluerunner::Stack;

/// An interactive shell that runs glue commands using a stack.
/// Commands are incrementally added and executed as they are passed
/// to cin from prompt.
pub struct Shell {
	/// The `Stack` instance of the shell.
	/// This instance lives for the entire `Shell` instance lifetime.
	stack: Stack,
}

impl Shell {
	/// Creates a new `Shell` instance with an empty `Stack`.
	pub fn new() -> Self {
		Shell {
			stack: Stack::new(),
		}
	}

	/// Start the prompt loop asynchronously. The loop executes a glue command
	/// at each iteration, or ends when a `None` is provided as command.
	///
	/// Note that a `None` command equals to a prompt with a `exit` or `quit`
	/// string: empty string will just be ignored and the loop will continue.
	pub async fn start(self: &mut Self) -> () {
		loop {
			// Get next command from prompt. Break loop if None.
			let glue_command = match prompt() {
				None => break,
				Some(x) => x,
			};

			if glue_command != "" {
				// Add command to stack, print error and skip loop iteration on error.
				match self.stack.push_runner_from_string(&glue_command, false) {
					Err(x) => {
						print_err(x);
						return;
					}
					_ => (),
				};

				// Execute the command and print result.
				match self.stack.execute_next().await {
					Err(x) => print_err(x),
					Ok(_) => println!("{}", self.stack.current().unwrap().result.as_ref().unwrap()),
				};
			}
		}
	}
}
