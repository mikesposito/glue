mod args;

use args::{command_args, Args};
use gluerunner::{heap, Runner};

#[tokio::main]
async fn main() {
	let args: Args = command_args();

	if args.file.is_none() && args.request.is_none() {
		glueshell::interactive().await;
		return;
	}

	let heap = heap();

	let mut glue_runner = match args.file {
		None => match Runner::from_string(&args.request.unwrap(), heap, args.verbose) {
			Err(x) => panic!("Error encountered while creating glue: {}", x),
			Ok(x) => x,
		},
		Some(x) => match Runner::from_file(&x, heap, args.verbose) {
			Err(x) => panic!("Error encountered while creating glue: {}", x),
			Ok(x) => x,
		},
	};

	match glue_runner.execute().await {
		Err(x) => panic!("Error encountered while executing requests: {}", x),
		Ok(_) => println!("{}", glue_runner.result.unwrap()),
	};
}
