use rand::prelude::random;

const OPEN_DELIMITER: char = '{';
const CLOSE_DELIMITER: char = '}';

#[derive(Debug, Clone)]
pub struct RequestNode {
	pub id: u32,
	pub command: String,
	pub predicate: String,
	pub method: String,
	pub url: String,
	pub result_selector: String,
	pub body: Option<RequestBody>,
	pub dependencies: Vec<RequestNode>,
	pub depth: usize,
	pub result: String,
}

#[derive(Debug, Clone)]
pub struct RequestBody {
	pub body_type: RequestBodyType,
	pub value: String,
}

#[derive(Debug, Clone)]
pub enum RequestBodyType {
	UNKNOWN,
	TEXT,
	JSON,
}

impl RequestNode {
	pub fn new(command: &String, depth: usize) -> Self {
		RequestNode {
			id: random(),
			command: String::from(command),
			predicate: String::from(""), // for root node command equals the predicate
			method: String::from(""),    // for root node command equals the predicate
			url: String::from(""),       // for root node command equals the predicate
			body: None,
			result_selector: String::from(""), // for root node command equals the predicate
			dependencies: Vec::new(),
			depth,
			result: String::from(""),
		}
	}

	/// Takes a String as input and returns a RequestNode 
	/// instance wrapped in Result. Returns an Err Result if
	/// something goes wrong with the parsing
	pub fn from_string(command: &String) -> Result<Self, String> {
		let mut root_node = RequestNode::new(command, 0);

		match root_node.build_tree_recursive() {
			Err(x) => Err(x),
			Ok(_) => Ok(root_node),
		}
	}

	/// Recursively parses the command up until a closing
	/// delimiter is encountered. It calls itself when a new
	/// open delimiter is encountered. Returns a wrapped usize
	/// to the function caller to allow it to determine at which
	/// point of the command the dependency was closed
	fn build_tree_recursive(self: &mut Self) -> Result<usize, String> {
		let mut close_dep_at: usize = 0;
		let mut skip_till: usize = 0;

		for (i, char) in &mut self.command.chars().enumerate() {
			// skip parsing if looping through dependency
			if skip_till > 0 && i <= skip_till + 1 {
				continue;
			}

			match char {
				OPEN_DELIMITER => {
					let mut dependency =
						RequestNode::new(&self.command[(i + 1)..].to_string(), self.depth + 1);

					match dependency.build_tree_recursive() {
						Err(x) => return Err(x),
						Ok(x) => skip_till = x + i,
					};

					self.dependencies.push(dependency);
					self.predicate.push_str("{}");
				}

				CLOSE_DELIMITER => {
					close_dep_at = i;
					// ignore everything after the closing delimiter
					break;
				}

				_ => self.predicate.push(char),
			}
		}

		match self.resolve_predicate() {
			Err(x) => Err(x),
			Ok(_) => Ok(close_dep_at),
		}
	}

	/// Reads the predicate and tries to resolve
	/// method, url and selector
	fn resolve_predicate(self: &mut Self) -> Result<(), String> {
		let mut predicate_parts = self.predicate.trim().split(' ');

		self.method = match predicate_parts.nth(0) {
			None => return Err(String::from("Failed to resolve method")),
			Some(x) => x.to_string(),
		};

		let resource = match predicate_parts.nth(0) {
			None => return Err(String::from("Failed to resolve request url")),
			Some(x) => x 
		};

		let mut resource_parts = resource.split('^');
		self.url = match resource_parts.nth(0) {
			None => return Err(String::from("Failed to resolve request url")),
			Some(x) => x.to_string(),
		};

		self.result_selector = match resource_parts.nth(0) {
			None => "".to_string(),
			Some(x) => x.to_string(),
		};

		Ok(())
	}

	pub fn print_info(self: &Self) -> () {
		println!("> [{}] {}", self.method.to_uppercase(), self.url);
	}
}

#[derive(Debug)]

/// GlueStack
pub struct GlueStack {
	pub layers: Vec<Vec<RequestNode>>,
	pub depth: usize,
	pub result: Option<String>,
}

impl GlueStack {
	fn new() -> Self {
		GlueStack {
			layers: vec![vec![]],
			depth: 0,
			result: None
		}
	}

	/// Creates a reversed GlueStack instance containing all 
	/// node requests in a root node
	pub fn from_root_node(root: &RequestNode) -> Self {
		let mut stack = GlueStack::new();

		stack.add_node_recursive(&root);
		stack.reverse();

		stack
	}

	/// Creates a reversed GlueStack instance containing all 
	/// node requests in a root node created from string
	pub fn from_string(command: &String) -> Result<Self, String> {
		match RequestNode::from_string(command) {
			Err(x) => Err(x),
			Ok(x) => Ok(GlueStack::from_root_node(&x)),
		}
	}

	/// Adds a node to the stack automatically determining 
	/// the layer it belongs to
	pub fn add_node(self: &mut Self, node: &RequestNode) -> () {
		if self.depth < node.depth {
			self.add_layer();
		}
		self.layers[self.depth].push(node.to_owned());
	}

	pub fn reverse(self: &mut Self) -> () {
		self.layers.reverse();
	}

	fn add_layer(self: &mut Self) -> () {
		self.layers.push(vec![]);
		self.depth += 1;
	}

	fn add_node_recursive(self: &mut Self, node: &RequestNode) -> () {
		self.add_node(node);
		for dep_node in &node.dependencies {
			self.add_node_recursive(dep_node);
		}
	}
}
