mod args;
pub mod glue;

use args::{parse_command_args, Args};
use glue::GlueStack;

#[tokio::main]
async fn main() {
	let args: Args = parse_command_args();

	let mut glue_stack = match GlueStack::from_string(&args.request) {
		Err(x) => panic!("Error encountered while creating glue: {}", x),
		Ok(x) => x,
	};

	match glue_stack.execute().await {
		Err(x) => panic!("Error encountered while executing requests: {}", x),
		Ok(_) => println!("\n\n{}", glue_stack.result.unwrap()),
	};
}
