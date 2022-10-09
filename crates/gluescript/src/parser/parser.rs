use crate::{GlueNode, ParserError};

use super::{expression::Expression, mask::Mask};

pub struct Parser {
	expressions: Vec<Expression>,
}

impl Parser {
	pub fn parse(value: String) -> Result<Self, ParserError> {
		let value = Mask::new(value);

		let expressions = value
			.masked()
			.split(';')
      .filter(|unmasked| unmasked.trim() != "")
			.map(|unmasked| Mask::derive(unmasked.trim().to_string(), &value))
			.map(|command_mask| Ok(Expression::from_mask(command_mask)?))
			.collect::<Result<Vec<Expression>, _>>()?;

		Ok(Parser { expressions })
	}

	pub fn to_nodes(self: &Self) -> Result<Vec<GlueNode>, ParserError> {
		self.expressions
			.iter()
			.map(|expression| Ok(expression.to_node(0)?))
			.collect::<Result<Vec<GlueNode>, _>>()
	}
}
