use crate::{DepMap, HeapMap, MuxNode, RequestError};
use gluescript::{constants, RequestBodyType};
use jsonpath_rust::JsonPathFinder;
use reqwest::Client;
use std::{error::Error, sync::Arc};

pub async fn execute_node(
	node: MuxNode,
	task_dependencies: DepMap,
	heap: HeapMap,
	log_info: bool,
) -> Result<(usize, String), String> {
	let mut w_node = node.lock().unwrap();

	if w_node.dependencies.len() > 0 {
		let dep_ids: Vec<u32> = w_node.dependencies.iter().map(|dep| dep.id).collect();

		for id in dep_ids.into_iter() {
			w_node.predicate = w_node.predicate.replacen(
				"{}",
				task_dependencies.lock().unwrap().get(&id).unwrap(),
				1,
			);
		}
	}

	match w_node.resolve_predicate() {
		Err(x) => return Err(x),
		_ => (),
	};

	if log_info {
		w_node.print_info();
	}

	let result = match w_node.method.as_str() {
		// we take the result from the heap if it's a saved variable
		constants::REQ => String::from(
			heap.lock()
				.unwrap()
				.get(&w_node.url)
				.expect(constants::ERR_UNRESOLVED_VAR),
		),
		// instead, an http request is fired
		_ => match send_http_request(Arc::clone(&node)).await {
			Err(x) => return Err(x.to_string()),
			Ok(x) => x,
		},
	};

	let is_root = w_node.depth == 0;

	w_node.result = match get_response_value(&w_node.result_selector, &result, !is_root, is_root) {
		Err(x) => return Err(x),
		Ok(x) => x,
	};

	let mut task_dependencies = task_dependencies.lock().unwrap();
	task_dependencies.insert(w_node.id, String::from(&w_node.result));
	drop(task_dependencies);

	if w_node.save_as.is_some() {
		let var_key = String::from(w_node.save_as.clone().unwrap().trim());
		let mut heap = heap.lock().unwrap();
		heap.insert(var_key, String::from(&w_node.result));
		drop(heap);
	}

	Ok((w_node.depth, String::from(&w_node.result)))
}

pub async fn send_http_request(node: MuxNode) -> Result<String, Box<dyn Error>> {
	let node = node.lock().unwrap();

	let client = match node.method.as_str() {
		constants::GET => Client::new().get(&node.url),
		constants::POST => Client::new().post(&node.url),
		constants::PUT => Client::new().put(&node.url),
		constants::PATCH => Client::new().patch(&node.url),
		constants::DELETE => Client::new().delete(&node.url),
		_ => {
			return Err(Box::new(RequestError(
				constants::ERR_UNKNOWN_METHOD.to_string(),
			)))
		}
	};

	let mut request = match &node.body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		},
	};

	if node.headers.is_some() {
		request = request.headers(node.headers.clone().unwrap());
	}

	let response = request.send().await?.text().await?;
	Ok(response)
}

fn get_response_value(
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
