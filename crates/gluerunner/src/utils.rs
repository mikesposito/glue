use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

/// Create a new heap map protected by `Arc<Mutex<T>>`
pub fn heap() -> Arc<Mutex<HashMap<String, String>>> {
	Arc::new(Mutex::new(HashMap::new()))
}
