mod args;

use args::{command_args, Args};
use gluerunner::{heap, Runner};
use glueshell::Shell;

#[tokio::main]
async fn main() {
	let args: Args = command_args();

	if args.file.is_none() && args.request.is_none() {
		// If no file and no request are provided, a new `Shell`
		// is instantiated to prompt command in loop.
		let mut shell = Shell::new();

		// Start the shell and block till finished.
		shell.start().await;
		return;
	}

	// Instantiate an `HashMap` protected by a `Mutex` to allow
	// request nodes to share data between each other from
	// different non-blocking threads
	let heap = heap();

	let mut glue_runner = match args.file {
		// Create `Runner` from command string if no file is provided
		None => match Runner::from_string(&args.request.unwrap(), heap, args.verbose) {
			Err(x) => panic!("Error encountered while creating glue: {}", x),
			Ok(x) => x,
		},

		// Or use a file content
		Some(x) => match Runner::from_file(&x, heap, args.verbose) {
			Err(x) => panic!("Error encountered while creating glue: {}", x),
			Ok(x) => x,
		},
	};

	// Execute command and print result. Panic on error.
	match glue_runner.execute().await {
		Err(x) => panic!("Error encountered while executing requests: {}", x),
		Ok(_) => println!("{}", glue_runner.result.clone().unwrap()),
	};
}
