use std::sync::{Arc, Mutex};

use crate::{constants, GlueNode};

use super::{errors::ParserError, mask::Mask, token::Token};

#[derive(Debug)]
pub struct Expression {
	raw: Mask,
	pub tokens: Vec<Token>,
	pub references: Vec<Expression>,
}

impl Expression {
	fn new(raw: Mask) -> Result<Self, ParserError> {
		let mut expression = Expression {
			raw,
			tokens: vec![],
			references: vec![],
		};

		expression.build_tree_recursive()?;
		expression.parse_raw_recursive()?;

		Ok(expression)
	}

	pub fn from_string(value: String) -> Result<Self, ParserError> {
		let value = Mask::new(value);
		Ok(Expression::new(value)?)
	}

	pub fn from_mask(value: Mask) -> Result<Self, ParserError> {
		Ok(Expression::new(value)?)
	}

	pub fn token_at(self: &Self, index: usize) -> Option<&Token> {
		self.tokens.iter().nth(index)
	}

	pub fn to_node(self: &Self, depth: usize) -> Result<GlueNode, ParserError> {
		let mut node = GlueNode::new(&self.raw.unmasked(), depth);

		self.tokens
			.iter()
			.map(|token| {
				Ok(match token {
					Token::Url(x) => node.url = x.value(),
					Token::Method(x) => node.method = x.value(),
					Token::Selector(x) => node.result_selector = x.value(),
					Token::SaveAs(x) => node.save_as = Some(x.value()),
					Token::BodyAttribute(x) => node.set_body_attribute(x.value()?),
					Token::BodyRaw(x) => node.set_body_raw(x.value()),
					Token::HeaderAttribute(x) => node.set_header(x.value()?)?,
				})
			})
			.collect::<Result<(), _>>()?;

		// Recursively add dependencies from expression references
		self.references
			.iter()
			.map(|reference| {
				Ok(node
					.dependencies
					.push(Arc::new(Mutex::new(reference.to_node(depth + 1)?))))
			})
			.collect::<Result<(), _>>()?;

		Ok(node)
	}

	fn build_tree_recursive(self: &mut Self) -> Result<usize, ParserError> {
		// This holds an index to the next dependency closing delimiter while
		// iterating over chars of `self.command`.
		// This index is used to skip parsing if iterating in a `command` substr
		// that is part of a dependency, because its parsing will be already done
		// by another iteration of `build_tree_recursive()`
		let mut skip_till: usize = 0;

		let mut parsed = String::new();

		// Parse the command, char by char.
		for (i, char) in &mut self.raw.masked().chars().enumerate() {
			// Skip parsing if cursor is in a dependency
			if skip_till > 0 && i <= skip_till + 1 {
				continue;
			}

			match char {
				// If an open delimiter is hit the control flow must be passed
				// to another iteration of the function, so the predicate
				// can be parsed for the dependency.
				constants::OPEN_DELIMITER => {
					// Create a new `GlueNode` instance from a `command`
					// based on a substring of `self.command`, starting from
					// the current cursor index. Depth is incrementally assigned.
					let dependency = Expression::new(Mask::derive(
						self.raw.masked()[(i + 1)..].to_string(),
						&self.raw,
					))?;

					// build_tree_recursive() is called for the newly created dependency
					// and its result is used to know where is the next closing delimiter.
					// Err is propagated on dependency parsing failure.
					skip_till = dependency.len() + i;

					// Dependency is pushed into the `dependencies` collection.
					self.references.push(dependency);

					// A `{}` is added to `self.predicate` so it will be possible
					// to replace it afterwards with the actual dependency
					// result.
					parsed.push_str("{}");
				}

				// If a closing delimiter is hit, the function is terminated
				// and the current cursor index is returned to the caller.
				constants::CLOSE_DELIMITER => {
					self.raw.assign(parsed);
					return Ok(i);
				}

				// In any other case, the char is added to `self.predicate`.
				_ => parsed.push(char),
			}
		}

		self.raw.assign(parsed);

		// If this statement is reached, then the `GlueNode`
		// that is executing this function is not a dependency
		// of another node, so 0 is returned.
		Ok(0)
	}

	fn parse_raw_recursive(self: &mut Self) -> Result<(), ParserError> {
    println!("'{}'", self.raw.masked());
		self.tokens = self
			.raw
			.masked()
			.split([' ', '\n', '\t'])
      .filter(|part| part.trim() != "")
			.map(|part| Mask::derive(part.trim().to_string(), &self.raw))
			.enumerate()
			.map(|(position, mask)| Ok(Token::new(mask, position)?))
			.collect::<Result<Vec<Token>, _>>()?;

		self.references
			.iter_mut()
			.map(|reference| Ok(reference.parse_raw_recursive()?))
			.collect::<Result<(), _>>()?;

		Ok(())
	}

	pub fn len(self: &Self) -> usize {
		self.raw.unmasked().len()
	}
}
