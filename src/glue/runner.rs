use super::constants;
use super::{RequestBody, RequestBodyType};
use jsonpath_rust::JsonPathFinder;
use reqwest::Client;
use reqwest::header::HeaderMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct RequestError(String);

impl Error for RequestError {}

impl fmt::Display for RequestError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "There is an error: {}", self.0)
	}
}

pub async fn http_request(
	method: &String,
	url: &String,
	headers: &Option<HeaderMap>,
	body: &Option<RequestBody>,
) -> Result<String, Box<dyn Error>> {
	let client = match method.as_str() {
		constants::GET => Client::new().get(url),
		constants::POST => Client::new().post(url),
		constants::PUT => Client::new().put(url),
		constants::PATCH => Client::new().patch(url),
		constants::DELETE => Client::new().delete(url),
		_ => {
			return Err(Box::new(RequestError(
				constants::ERR_UNKNOWN_METHOD.to_string(),
			)))
		}
	};

	let mut request = match body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		},
	};

	request = match headers {
		None => request,
		Some(x) => request.headers(x.to_owned()),
	};

	let response = request.send().await?.text().await?;
	Ok(response)
}

pub fn get_response_value(
	path: &String,
	response: &String,
	just_first_slice_value: bool,
	pretty: bool,
) -> Result<String, String> {
	// No path provided so we return response
	// as is
	if path.len() == 0 {
		return Ok(String::from(response));
	}

	// Path has been provided so we suppose to have
	// a json response
	let json_selectable = match JsonPathFinder::from_str(&response[..], &path[..]) {
		Err(x) => return Err(x),
		Ok(x) => x,
	};

	// JSONPath returns an array with results as default
	// but sometimes what we need from response
	// is just a single string value
	if just_first_slice_value {
		return match json_selectable.find_slice().to_vec()[0].as_str() {
			None => Err(format!("Could not select value to use with this selector: \n{path} \non this response: \n{response}")),
			Some(x) => Ok(String::from(x)),
		};
	}

	// return prettified result string
	if pretty {
		return match serde_json::to_string_pretty(json_selectable.find_slice().as_slice()) {
			Err(x) => Err(x.to_string()),
			Ok(x) => Ok(x),
		};
	}

	// ..or non-prettified
	match serde_json::to_string(json_selectable.find_slice().as_slice()) {
		Err(x) => Err(x.to_string()),
		Ok(x) => Ok(x),
	}
}
