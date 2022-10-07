use std::{sync::{Arc, Mutex}, collections::HashMap};

pub fn heap() -> Arc<Mutex<HashMap<String, String>>> {
  Arc::new(Mutex::new(HashMap::new()))
}