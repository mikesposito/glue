use colored::*;
use gluerunner::RunnerStack;
use std::io::{stdin, stdout, Write};

pub async fn interactive() -> () {
	let mut stack = RunnerStack::new();

	loop {
		let command = prompt();

		if command.is_none() {
			break;
		}

		let glue_command = command.unwrap();

		if glue_command != "" {
			match stack.push_runner_from_string(&glue_command, false) {
				Err(x) => {
					print_err(x);
					return;
				}
				_ => (),
			};

			match stack.execute_next().await {
				Err(x) => print_err(x),
				Ok(_) => println!("{}", stack.current().unwrap().result.as_ref().unwrap()),
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
