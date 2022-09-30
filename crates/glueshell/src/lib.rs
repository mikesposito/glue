use colored::*;
use gluerunner::Runner;
use std::io::{stdin, stdout, Write};

pub async fn interactive() -> () {
	loop {
		let command = prompt();

		match command {
      Some(x ) if x == "" => (), 
			Some(x) => {
				let mut runner = match Runner::from_string(&x, false) {
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
			_ => break,
		};
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
