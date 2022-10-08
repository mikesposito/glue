use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RequestBody {
	pub body_type: RequestBodyType,
	pub value: HashMap<String, String>,
	pub raw: String,
}

#[derive(Debug, Clone)]
pub enum RequestBodyType {
	JSON,
	FORM,
	ARBITRARY,
}

impl RequestBody {
	pub fn new(
		body_type: RequestBodyType,
		value: Option<HashMap<String, String>>,
		raw: Option<String>,
	) -> Self {
		match body_type {
			RequestBodyType::ARBITRARY => RequestBody {
				body_type,
				value: HashMap::new(),
				raw: raw.unwrap(),
			},
			_ => RequestBody {
				body_type,
				value: value.unwrap(),
				raw: "".to_string(),
			},
		}
	}
}
