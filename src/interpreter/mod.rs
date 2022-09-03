use crate::parser::lib::{RequestNode, get_ordered_dependencies_list};
use crate::runner::run;

pub fn interpret(request_node: &mut RequestNode) -> () {
	let mut dependencies: Vec<_> = get_ordered_dependencies_list(request_node);

	run(&mut dependencies);
}
