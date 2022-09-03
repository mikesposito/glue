pub mod lib;
use lib::RequestNode;

const OPEN_DELIMITER: char = '{';
const CLOSE_DELIMITER: char = '}';

pub fn parse(command: String) -> RequestNode {
	let mut root_node = RequestNode::new(command, 0);
	// # Letter by letter, start from tree root predicate, and build nested dependencies
	build_tree(&mut root_node);

	root_node
}

pub fn build_tree(node: &mut RequestNode) -> usize {
	let mut close_dep_at: usize = 0;
	let mut skip_till: usize = 0;

	for (i, char) in node.clone().command.chars().enumerate() {
		// skip parsing if looping through dependency
		if skip_till > 0 && i <= skip_till + 1 {
			continue;
		}

		match char {
			OPEN_DELIMITER => {
				node.open_dependency(i);
				let dep_index: usize = node.dependencies.len() - 1;
				skip_till = build_tree(&mut node.dependencies[dep_index]) + i;
				node.close_dependency();
			}

			CLOSE_DELIMITER => {
				close_dep_at = i;
				// ignore everything after the closing delimiter
				break;
			}

			_ => node.predicate.push(char),
		}
	}

	close_dep_at
}
