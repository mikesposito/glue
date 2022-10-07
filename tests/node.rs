use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use gluescript::node::GlueNode;

const SIMPLE_COMMAND: &str = "get http://example.com";
const WITH_SELECTOR_NESTED_COMMAND: &str = "get http://example.com/{post http://test.com^$.id}/";

fn get_node(command: String) -> GlueNode {
	GlueNode::from_string(&command).unwrap()
}

fn get_dep_map() -> Arc<Mutex<HashMap<u32, String>>> {
	Arc::new(Mutex::new(HashMap::new()))
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

	node.dependencies[0].resolve_predicate().unwrap();
	assert_eq!(node.dependencies[0].url, "http://test.com");
	assert_eq!(node.dependencies[0].method, "post");
	assert_eq!(node.dependencies[0].result_selector, "$.id");

	let deps = get_dep_map();

	let mut w_deps = deps.lock().unwrap();
	w_deps.insert(node.dependencies[0].id, "test".to_string());
	drop(w_deps);

	node.resolve_dependencies(Arc::clone(&deps)).unwrap();
	node.resolve_predicate().unwrap();

	assert_eq!(node.url, "http://example.com/test/".to_string());
}
