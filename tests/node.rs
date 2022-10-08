use gluescript::node::GlueNode;

const SIMPLE_COMMAND: &str = "get http://example.com";
const SIMPLE_COMMAND_WITH_BODY: &str = r#"get http://example.com~username=admin~password="xxx-?|>^-*~xx""#;
const SIMPLE_COMMAND_WITH_HEADERS: &str =
	r#"get http://example.com*Authorization="Bearer xxx-?|>^-*~xx""#;
const WITH_SELECTOR_NESTED_COMMAND: &str = "get http://example.com/{post http://test.com^$.id}/";

fn get_node(command: String) -> GlueNode {
	GlueNode::from_string(&command).unwrap()
}

#[test]
fn it_parses_simple_request() {
	let node = get_node(SIMPLE_COMMAND.to_string());
	assert_eq!(node.predicate, SIMPLE_COMMAND.to_string());
}

#[test]
fn it_resolves_method_correctly() {
	let mut node = get_node(SIMPLE_COMMAND.to_string());
	node.resolve_predicate().unwrap();
	assert_eq!(node.method, "get".to_string());
}

#[test]
fn it_resolves_url_correctly() {
	let mut node = get_node(SIMPLE_COMMAND.to_string());
	node.resolve_predicate().unwrap();
	assert_eq!("http://example.com".to_string(), node.url);
}

#[test]
fn it_resolves_dependency_correctly() {
	let mut node = get_node(WITH_SELECTOR_NESTED_COMMAND.to_string());
	assert_eq!(node.dependencies.len(), 1);

	let mut w_dep = node.dependencies[0].lock().unwrap();
	w_dep.resolve_predicate().unwrap();
	assert_eq!(w_dep.url, "http://test.com");
	assert_eq!(w_dep.method, "post");
	assert_eq!(w_dep.result_selector, "$.id");

	w_dep.result = "test".to_string();
	drop(w_dep);

	node.resolve_dependencies();
	node.resolve_predicate().unwrap();

	assert_eq!(node.url, "http://example.com/test/".to_string());
}

#[test]
fn it_resolves_body_correctly() {
	let mut node = get_node(SIMPLE_COMMAND_WITH_BODY.to_string());
	node.resolve_predicate().unwrap();
	assert_eq!(
		"admin",
		node.body
			.clone()
			.unwrap()
			.value
			.get(&"username".to_string())
			.unwrap()
	);
	assert_eq!(
		"xxx-?|>^-*~xx",
		node.body
			.clone()
			.unwrap()
			.value
			.get(&"password".to_string())
			.unwrap()
	);
}

#[test]
fn it_resolves_headers_correctly() {
	let mut node = get_node(SIMPLE_COMMAND_WITH_HEADERS.to_string());
	node.resolve_predicate().unwrap();
	assert_eq!(
		"Bearer xxx-?|>^-*~xx",
		node.headers
			.clone()
			.unwrap()
			.get(&"Authorization".to_string())
			.unwrap()
	);
}
