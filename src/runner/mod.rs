mod lib;

use crate::parser::{GlueStack, RequestNode};
use lib::{get, get_response_value};
use std::collections::HashMap;

pub async fn run_stack(stack: &mut GlueStack) -> Result<(), String> {
	let mut dependencies_resolutions: HashMap<u32, String> = HashMap::new();

	for layer in &mut stack.layers {
		let mut tasks = vec![];

		for (i, request) in layer.into_iter().enumerate() {
			let task_dependencies = dependencies_resolutions.clone();
			let mut node = request.clone();

			tasks.push(tokio::spawn(async move {
				if node.dependencies.len() > 0 {
					for dependency in &node.dependencies {
						let dependency_result = task_dependencies.get(&dependency.id).unwrap();
						node.url = node.url.replacen("{}", dependency_result, 1);
					}
				}

				node.print_info();

				node.result = match run_request(&node).await {
					Err(x) => return Err(x),
					Ok(x) => x,
				};

				Ok((i, node))
			}));
		}

		for task in tasks {
			match task.await {
				Err(x) => return Err(x.to_string()),
				Ok(result) => {
					match result {
						Err(x) => return Err(x),
						Ok((i, node)) => {
							if node.depth == 0 {
								stack.result = Some(String::from(&node.result));
							}
							dependencies_resolutions.insert(node.id, String::from(&node.result));
							layer[i] = node;
						}
					};
				}
			}
		}
	}

	Ok(())
}

pub async fn run_request(request: &RequestNode) -> Result<String, String> {
	let result = match get(&request.url).await {
		Err(x) => return Err(x.to_string()),
		Ok(x) => x,
	};

	let is_root = request.depth == 0;

	match get_response_value(&request.result_selector, &result, !is_root, is_root) {
		Err(x) => Err(x),
		Ok(x) => Ok(x),
	}
}
