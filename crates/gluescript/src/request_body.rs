use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RequestBody {
	pub body_type: RequestBodyType,
	pub value: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum RequestBodyType {
	JSON,
	FORM,
}

impl RequestBody {
	pub fn new(body_type: RequestBodyType, value: HashMap<String, String>) -> Self {
		RequestBody { body_type, value }
	}
}
