mod args;
mod interpreter;
mod parser;
mod runner;

use args::{parse_command_args, Args};
use interpreter::interpret;
use parser::lib::RequestNode;
use parser::parse;

fn main() {
	let args: Args = parse_command_args();
	
	let mut request_node: RequestNode = parse(args.request);

	interpret(&mut request_node);
}