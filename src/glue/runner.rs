use super::{RequestBody, RequestBodyType};
use jsonpath_rust::JsonPathFinder;

pub async fn get(url: &String) -> Result<String, Box<dyn std::error::Error>> {
	let resp = reqwest::get(url).await?.text().await?;
	Ok(resp)
}

pub async fn post(url: &String, body: &Option<RequestBody>) -> Result<String, Box<dyn std::error::Error>> {
	let client = reqwest::Client::new().post(url);
	let request = match body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		}
	};

	let response = request.send().await?.text().await?;
	Ok(response)
}

pub async fn put(url: &String, body: &Option<RequestBody>) -> Result<String, Box<dyn std::error::Error>> {
	let client = reqwest::Client::new().put(url);
	let request = match body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		}
	};

	let response = request.send().await?.text().await?;
	Ok(response)
}

pub async fn patch(url: &String, body: &Option<RequestBody>) -> Result<String, Box<dyn std::error::Error>> {
	let client = reqwest::Client::new().patch(url);
	let request = match body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		}
	};

	let response = request.send().await?.text().await?;
	Ok(response)
}

pub async fn delete(url: &String, body: &Option<RequestBody>) -> Result<String, Box<dyn std::error::Error>> {
	let client = reqwest::Client::new().delete(url);
	let request = match body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		}
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
