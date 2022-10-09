use crate::utils::{deserialize, serialize};

#[derive(Debug, Clone)]
pub struct Mask {
	masked: String,

	raw: String,

	components: Vec<String>,
}

impl Mask {
	pub fn new(raw: String) -> Self {
		let (masked, components) = serialize(raw.clone());

		Mask {
			raw: raw.clone(),
			masked,
			components,
		}
	}

	pub fn derive(value: String, from_mask: &Mask) -> Self {
		let (masked, components) = serialize(from_mask.unmask_part(value.clone()));

		Mask {
			raw: value,
			masked,
			components,
		}
	}

	pub fn masked(self: &Self) -> String {
		String::from(&self.masked)
	}

	pub fn unmasked(self: &Self) -> String {
		String::from(&self.raw)
	}

	pub fn unmask_part(self: &Self, part: String) -> String {
		deserialize(part, &self.components)
	}

	pub fn assign(self: &mut Self, value: String) -> () {
		self.raw = value;
		self.refresh_mask();
	}

	pub fn push(self: &mut Self, char: char) {
		self.raw.push(char);
		self.refresh_mask();
	}

	pub fn push_str(self: &mut Self, str: &str) {
		self.raw.push_str(str);
		self.refresh_mask();
	}

	fn refresh_mask(self: &mut Self) -> () {
		let (masked, components) = serialize(self.raw.clone());

		self.masked = masked;
		self.components = components;
	}
}
