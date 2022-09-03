mod lib;

use crate::parser::lib::RequestNode;
use lib::{get, get_response_value};
use std::collections::HashMap;

pub fn run(requests: &mut Vec<RequestNode>) -> () {
	let mut dependencies_resolutions: HashMap<u32, String> = HashMap::new();

	for request in requests {
		let mut parts = request.predicate.split(' ');
		let method = parts.nth(0).unwrap().to_string();
		let mut url = parts.nth(0).unwrap().to_string();

		if request.dependencies.len() > 0 {
			for dependency in &request.dependencies {
				let dependency_result = dependencies_resolutions.get(&dependency.id).unwrap();
				// println!("\t{}\n", dependency_result);
				url = url.replacen("{}", &dependency_result, 1);
			}
		}

		let url_parts: Vec<&str> = url.split('^').collect();

		let response_usable_value = match url_parts.len() > 1 {
			true => url_parts[1].to_string(),
			false => "".to_string(),
		};

		println!(
			"> [{}] {}",
			method.to_uppercase(),
			&url_parts[0].to_string()
		);

		let result = get(&url_parts[0].to_string());

		if !result.is_ok() {
			println!("{}", result.unwrap_err());
			return;
		}

		let is_root = request.depth == 0;
		let response_value = get_response_value(
			&response_usable_value,
			&result.unwrap().text().unwrap(),
			!is_root,
			is_root,
		);

		if request.depth == 0 {
			request.result = response_value;
			println!("{}", request.result);
		} else {
			let dep_return_value = response_value;
			dependencies_resolutions.insert(request.id, dep_return_value);
		}
	}
}
