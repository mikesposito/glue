use gluescript::{
	parser::token::{Attribute, Value},
	Expression, Parser, Token,
};

const SIMPLE_COMMAND: &str =
	"get http://example.com ^$.a.selector ~body=example *header=example >save_example";

const WITH_SELECTOR_NESTED_COMMAND: &str = "get http://example.com/{post http://test.com ^$.id}/";

#[test]
fn it_should_create_expression_from_string() {
	let expression = Expression::from_string(SIMPLE_COMMAND.to_string()).unwrap();

	assert_eq!(expression.tokens.len(), 6);
	assert_eq!(
		expression.token_at(0).unwrap(),
		&Token::Method(Value::new("get".to_string()))
	);
	assert_eq!(
		expression.token_at(1).unwrap(),
		&Token::Url(Value::new("http://example.com".to_string()))
	);
	assert_eq!(
		expression.token_at(2).unwrap(),
		&Token::Selector(Value::new("$.a.selector".to_string()))
	);
	assert_eq!(
		expression.token_at(3).unwrap(),
		&Token::BodyAttribute(Attribute::new("body=example".to_string()))
	);
	assert_eq!(
		expression.token_at(4).unwrap(),
		&Token::HeaderAttribute(Attribute::new("header=example".to_string()))
	);
	assert_eq!(
		expression.token_at(5).unwrap(),
		&Token::SaveAs(Value::new("save_example".to_string()))
	);
}

#[test]
fn it_should_correctly_create_node() {
	let nodes = Parser::parse(SIMPLE_COMMAND.to_string())
		.unwrap()
		.to_nodes()
		.unwrap();

	assert_eq!(nodes.len(), 1);
	assert_eq!(nodes[0].method, "get".to_string());
	assert_eq!(nodes[0].url, "http://example.com".to_string());
	assert_eq!(nodes[0].result_selector, "$.a.selector".to_string());
	assert_eq!(
		nodes[0].save_as.clone().unwrap(),
		"save_example".to_string()
	);
	assert_eq!(
		nodes[0]
			.body
			.clone()
			.unwrap()
			.value
			.get(&"body".to_string())
			.unwrap(),
		&"example".to_string()
	);
}

#[test]
fn it_should_correctly_parse_references() {
	let nodes = Parser::parse(WITH_SELECTOR_NESTED_COMMAND.to_string())
		.unwrap()
		.to_nodes()
		.unwrap();

	assert_eq!(nodes[0].dependencies.len(), 1);
	assert_eq!(
		nodes[0].dependencies[0].lock().unwrap().url,
		"http://test.com".to_string()
	);
}
