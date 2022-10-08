mod args;

use args::{command_args, Args};
use glueshell::Shell;

#[tokio::main]
async fn main() {
	let args: Args = command_args();

	// Fresh instance on `glueshell` instantiated to be ready
	// to run request, file or start interactive mode.
	let mut shell = Shell::new(args.verbose);

	if args.file.is_none() && args.request.is_none() {
		// Start the shell in interactive and block till finished
		// if no file or request are provided.
		shell.start().await;
		return;
	}

	match args.file {
		// Create `Runner` from command string if no file is provided
		None => shell
			.command(args.request.unwrap())
			.expect("Error encountered while running command."),

		// Or use a file content
		Some(x) => shell
			.load_file(x)
			.expect("Error encountered while loading file content."),
	};

	// Execute command and print result. Panic on error.
	match shell.execute_all().await {
		Err(x) => panic!("Error encountered while executing requests: {}", x),
		_ => (),
	};
}
