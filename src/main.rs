mod args;
mod parser;
mod runner;

use args::{parse_command_args, Args};
use parser::GlueStack;

use crate::runner::run_stack;

#[tokio::main]
async fn main() {
	let args: Args = parse_command_args();

	// let mut request_node = RequestNode::from_string(&args.request).unwrap();

	// interpret(&mut request_node);

	let mut glue_stack = match GlueStack::from_string(&args.request) {
		Err(x) => panic!("Error encountered while creating glue: {}", x),
		Ok(x) => x,
	};

	match run_stack(&mut glue_stack).await {
		Err(x) => panic!("Error encountered while executing requests: {}", x),
		Ok(_) => println!("\n\n{}", glue_stack.result.unwrap()),
	};

	//run_stack(&mut glue_stack).await.unwrap();
}
