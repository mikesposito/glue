use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

pub fn heap() -> Arc<Mutex<HashMap<String, String>>> {
	Arc::new(Mutex::new(HashMap::new()))
}
