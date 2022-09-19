use clap::Parser;
use clap::CommandFactory;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	pub request: Option<String>,

	#[clap(short, long, value_parser)]
	pub file: Option<String>
}

pub fn parse_command_args() -> Args {
	Args::parse()
}

pub fn print_help() {
	let mut cmd = Args::command();
	match cmd.print_help() {
		_ => ()
	}
}