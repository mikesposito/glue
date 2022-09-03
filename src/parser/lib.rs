use rand::prelude::random;

#[derive(Debug, Clone)]
pub struct RequestNode {
	pub id: u32,
	pub command: String,
	pub predicate: String,
	pub dependencies: Vec<RequestNode>,
	pub depth: usize,
	pub result: String,
}

impl RequestNode {
	pub fn new(command: String, depth: usize) -> Self {
		RequestNode {
			id: random(),
			command: String::from(&command),
			predicate: String::from(""), // for root node command equals the predicate
			dependencies: Vec::new(),
			depth,
			result: String::from(""),
		}
	}

	pub fn open_dependency(self: &mut Self, at_char: usize) -> () {
		let dependency =
			RequestNode::new(self.command[(at_char + 1)..].to_string(), self.depth + 1);
		// println!("[{}] Created dependency with command {}", dependency.depth, dependency.command);
		self.dependencies.push(dependency);
	}

	pub fn close_dependency(self: &mut Self) -> () {
		self.predicate.push_str("{}");
	}

	#[allow(dead_code)]
	pub fn print(node: &RequestNode) -> () {
		println!("Depth {}: {}", node.depth, node.predicate);
		for dependency in &node.dependencies {
			RequestNode::print(dependency);
		}
	}
}

pub fn get_ordered_dependencies_list(node: &RequestNode) -> Vec<RequestNode> {
	let mut dep_list: Vec<RequestNode> = vec![];

	dep_list.push(node.clone());

	// println!("get_ordered_dependencies_list ({:?})", node);
	for dependency in node.clone().dependencies {
		match dependency.dependencies.len() {
			x if x > 0 => dep_list.extend_from_slice(&get_ordered_dependencies_list(&dependency)),
			_ => {
				dep_list.push(dependency);
			}
		}
		// println!("{:#?}", dep_list);
	}

	dep_list.sort_unstable_by(|a, b| b.depth.cmp(&a.depth));
	dep_list
	// dep_list.iter().map(|dep| dep.predicate).collect()
}

#[allow(dead_code)]
pub fn get_predicates(dependencies: &Vec<RequestNode>) -> Vec<String> {
	dependencies
		.iter()
		.map(|dep| dep.clone().predicate)
		.collect()
}