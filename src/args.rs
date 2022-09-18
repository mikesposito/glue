use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	pub request: String,
	
}

pub fn parse_command_args() -> Args {
	Args::parse()
}
