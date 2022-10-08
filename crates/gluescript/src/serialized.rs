use crate::utils::{deserialize, serialize};

#[derive(Debug, Clone)]
pub struct Serialized {
	serialized: String,

	raw: String,

	components: Vec<String>,
}

impl Serialized {
	pub fn new(raw: String) -> Self {
		let (serialized, components) = serialize(raw.clone());

		Serialized {
			raw: raw.clone(),
			serialized,
			components,
		}
	}

	pub fn serialized(self: &Self) -> String {
		String::from(&self.serialized)
	}

	pub fn deserialized(self: &Self) -> String {
		String::from(&self.raw)
	}

	pub fn deserialize_part(self: &Self, part: String) -> String {
		deserialize(part, &self.components)
	}
}
