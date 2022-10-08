use colored::*;
use gluerunner::Stack;
use std::io::{stdin, stdout, Write};

/// An interactive shell that runs glue commands using a stack.
/// Commands are incrementally added and executed as they are passed
/// to cin from prompt.
pub struct Shell {
	/// The `Stack` instance of the shell.
	/// This instance lives for the entire `Shell` instance lifetime.
	pub stack: Stack,

	/// Collection of all previous run commands.
	history: Vec<String>,

	/// Verbose mode toggle
	verbose: bool,
}

impl Shell {
	/// Creates a new `Shell` instance with an empty `Stack`.
	pub fn new(verbose: bool) -> Self {
		Shell {
			stack: Stack::new(),
			history: vec![],
			verbose,
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
			let glue_command = match self.prompt() {
				None => break,
				Some(x) => x,
			};

			if glue_command != "" {
				// Save command to history.
				self.history.push(glue_command.clone());

				// Add command to stack, print error and skip loop iteration on error.
				match self.stack.push_runner_from_string(&glue_command, false) {
					Err(x) => {
						self.print_err(x);
						return;
					}
					_ => (),
				};

				// Execute the command and print result.
				match self.stack.execute_next().await {
					Err(x) => self.print_err(x),
					Ok(_) => println!("{}", self.stack.current().unwrap().result.as_ref().unwrap()),
				};
			}
		}
	}

	/// Load content of a file into the execution stack.
	pub fn load_file(self: &mut Self, path: String) -> Result<(), String> {
		self.stack.push_from_file(path, self.verbose)?;
		Ok(())
	}

	/// Execute all runners in the stack consecutively.
	pub async fn execute_all(self: &mut Self) -> Result<(), String> {
		self.stack.execute_all().await?;
		Ok(())
	}

	/// Add command to execution stack.
	pub fn command(self: &mut Self, command: String) -> Result<(), String> {
		self.stack.push_runner_from_string(&command, self.verbose)?;
		Ok(())
	}

	/// Command prompt from stdin.
	/// Return `None` if "exit" or "quit" are returned to stdin.
	fn prompt(self: &Self) -> Option<String> {
		let mut line = String::new();

		print!("{} ", "glue >".green());

		// Flush stdout to remove newline after prompt.
		stdout().flush().unwrap();

		// Read line from stdin.
		stdin()
			.read_line(&mut line)
			.expect("Error: Could not read a line");

		// Return `None` if "quit" or "exit" provided.
		// Otherwise return the whole `Some(line)`
		match line.trim().to_string() {
			x if x == "exit" || x == "quit" => None,
			x => Some(x),
		}
	}

	/// Print an error.
	fn print_err(self: &Self, err: String) -> () {
		println!("{} {}", "glue >".red(), err.red());
	}
}
