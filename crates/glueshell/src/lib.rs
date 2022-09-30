use colored::*;
use gluerunner::Runner;
use std::io::{stdin, stdout, Write};

pub async fn interactive() -> () {
	loop {
		let command = prompt();

		if command.is_none() {
			break;
		}

		let glue_command = command.unwrap();

		if glue_command != "" {
			let mut runner = match Runner::from_string(&glue_command, false) {
				Err(x) => {
					print_err(x);
					return;
				}
				Ok(x) => x,
			};

			match runner.execute().await {
				Err(x) => print_err(x),
				Ok(_) => println!("{}", runner.result.unwrap()),
			};
		}
	}
}

fn prompt() -> Option<String> {
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

fn print_err(err: String) -> () {
	println!("{} {}", "glue >".red(), err.red());
}
