mod args;
pub mod glue;

use args::{command_args, print_help, Args};
use glue::GlueStack;

#[tokio::main]
async fn main() {
	let args: Args = command_args();

	if args.file.is_none() && args.request.is_none() {
		print_help();
		return;
	}

	let mut glue_stack = match args.file {
		None => match GlueStack::from_string(&args.request.unwrap()) {
			Err(x) => panic!("Error encountered while creating glue: {}", x),
			Ok(x) => x,
		},
		Some(x) => match GlueStack::from_file(&x) {
			Err(x) => panic!("Error encountered while creating glue: {}", x),
			Ok(x) => x,
		},
	};

	match glue_stack.execute().await {
		Err(x) => panic!("Error encountered while executing requests: {}", x),
		Ok(_) => println!("{}", glue_stack.result.unwrap()),
	};
}
