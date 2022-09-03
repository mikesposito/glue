use reqwest::blocking::Response;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use jsonpath_rust::JsonPathFinder;

pub fn get(url: &String) -> Result<Response, Box<dyn std::error::Error>> {
	let resp = reqwest::blocking::get(url)?;
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
) -> String {
	match path.len() > 0 {
		true => {
			let json_selectable = JsonPathFinder::from_str(&response[..], &path[..]).unwrap();
			match just_first_slice_value {
				true => String::from(json_selectable.find_slice().to_vec()[0].as_str().unwrap()),
				false => match pretty {
					true => serde_json::to_string_pretty(json_selectable.find_slice().as_slice())
						.unwrap(),
					false => {
						serde_json::to_string(json_selectable.find_slice().as_slice()).unwrap()
					}
				},
			}
		},
		false => String::from(response),
	}
}