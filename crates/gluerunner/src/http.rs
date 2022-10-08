use crate::{HeapMap, MuxNode, RequestError};
use gluescript::{constants, RequestBodyType};
use jsonpath_rust::JsonPathFinder;
use reqwest::Client;
use std::{error::Error, sync::Arc};

/// Executes http call for a specific `GlueNode` behind an `Arc<Mutex<T>>` using
/// provided dependencies and heap `HashMap`.
pub async fn execute_node(
	node: MuxNode,
	heap: HeapMap,
	log_info: bool,
) -> Result<(), String> {
	// Acquire write lock on `GlueNode` mutex.
	let mut w_node = node.lock().unwrap();

	// Predicate can now be resolved as `GlueNode` should have all dependencies
	// resolved.
	match w_node.resolve_predicate() {
		Err(x) => return Err(x),
		_ => (),
	};

	// If verbose mode is enabled, we print the http request that is about
	// to be fired
	if log_info {
		w_node.print_info();
	}

	// Clone the method string from the node
	let method = String::from(&w_node.method);

	// Release the lock on `node` to allow http_request to run.
	drop(w_node);

	// Get `GlueNode` result executing http request or read from the heap
	let result = match method.as_str() {
		// Take the result from the heap if it's a saved variable
		constants::REQ => {
			let r_node = node.lock().unwrap();

			String::from(
				heap.lock()
					.unwrap()
					.get(&r_node.url)
					.expect(constants::ERR_UNRESOLVED_VAR),
			)
		}

		// Or with other methods, an http request is fired
		_ => match send_http_request(Arc::clone(&node)).await {
			Err(x) => return Err(x.to_string()),
			Ok(x) => x,
		},
	};

	// Lock writable node again to continue operations on it.
	let mut w_node = node.lock().unwrap();

	// The `GlueNode` is considered to be root if its depth is 0
	let is_root = w_node.depth == 0;

	// Select the response value based on the provided selector.
	// Get the whole response if no selector is provided.
	w_node.result = match get_response_value(&w_node.result_selector, &result, !is_root, is_root) {
		Err(x) => return Err(x),
		Ok(x) => x,
	};

	// If `save_as` has a value, then `result` value is saved into heap with
	// `save_as` as key and `result` as value.
	// Subsequent runs will be able to reuse this result from heap.
	if w_node.save_as.is_some() {
		// Get key to be used from `save_as` property
		let var_key = String::from(w_node.save_as.clone().unwrap().trim());

		// Acquire mutable lock on heap, as it might be shared by other `Runner` too.
		let mut heap = heap.lock().unwrap();

		heap.insert(var_key, String::from(&w_node.result));

		// Release immediately the lock to avoid useless blocks for
		// other threads.
		drop(heap);
	}

	Ok(())
}

/// Executes HTTP request declared in `node`.
///
/// `node` must be already full resolved.
pub async fn send_http_request(node: MuxNode) -> Result<String, Box<dyn Error>> {
	// Acquire read lock on `GlueNode` mutex.
	let node = node.lock().unwrap();

	// Build request starting from requested method.
	// Fail if method is unrecognized.
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

	// Append `GlueNode` body to request body in json or form
	// mode.
	let mut request = match &node.body {
		None => client,
		Some(body_map) => match body_map.body_type {
			RequestBodyType::JSON => client.json(&body_map.value),
			RequestBodyType::FORM => client.form(&body_map.value),
		},
	};

	// Append `GlueNode` headers to request headers.
	if node.headers.is_some() {
		request = request.headers(node.headers.clone().unwrap());
	}

	// Fire http request
	let response = request.send().await?.text().await?;
	Ok(response)
}

/// Select JSON response value with a JSONPath selector
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
