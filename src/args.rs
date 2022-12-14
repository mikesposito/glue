use clap::Parser;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	pub request: Option<String>,

	#[clap(short, long, value_parser)]
	pub file: Option<String>,

	#[clap(short, long)]
	pub verbose: bool,
}

pub fn command_args() -> Args {
	Args::parse()
}
