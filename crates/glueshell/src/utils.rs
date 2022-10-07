use colored::*;
use std::io::{stdin, stdout, Write};

pub(crate) fn prompt() -> Option<String> {
	let mut line = String::new();

	print!("{} ", "glue >".green());
	stdout().flush().unwrap();

	stdin()
		.read_line(&mut line)
		.expect("Error: Could not read a line");

	match line.trim().to_string() {
		x if x == "exit" || x == "quit" => None,
		x => Some(x),
	}
}

pub(crate) fn print_err(err: String) -> () {
	println!("{} {}", "glue >".red(), err.red());
}