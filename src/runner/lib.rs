use jsonpath_rust::JsonPathFinder;
use reqwest::header::{HeaderValue, CONTENT_TYPE};
use reqwest::Response;

pub async fn get(url: &String) -> Result<String, Box<dyn std::error::Error>> {
	let resp = reqwest::get(url).await?.text().await?;
	Ok(resp)
}

#[allow(dead_code)]
pub fn is_json_response(response: &Response) -> bool {
	let headers = response.headers();
	headers.get(CONTENT_TYPE) == Some(&HeaderValue::from_static("application/json"))
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
			None => {
				let mut error = "Could not select value to use with this selector: ".to_string();
				error.push_str(path);
				error.push_str("\n on this response: \n");
				error.push_str(response);
				Err(error)
			},
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
