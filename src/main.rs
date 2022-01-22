use std::env;
mod evernote2turtl;


fn parse_arg(args: &[String]) -> (&str, u32) {
	if args.len() < 3 {
        println!("Usage: {} [zip file or unzipped directory path] [numeric user ID]\n",
				args[0]);
		println!("- To find your 'numeric user ID' generate a Turtl backup");
        std::process::exit(0);
	}

	let user_id = match args[2].parse::<u32>() {
		Ok(v) => v,
		Err(e) => {
			println!("Failed to parse numeric user id: {}", e);
			std::process::exit(0);
		},
	};

	(args[1].as_str(), user_id)
}

fn main() {
    let args: Vec<String> = env::args().collect();
	let (complete_path, user_id) = parse_arg(&args);
	println!("{}", complete_path);
	let (_path, extension) = match complete_path.rsplit_once(".") {
		None => (complete_path, ""),
		Some(t) => t,
	};
	if extension == ".zip" {
        // Zip file
        let j = evernote2turtl::create_turtl_backup_from_zipfile(complete_path, user_id).unwrap();
        println!("{:#}", j);
    } else {
        // unzipped directory
        let j = evernote2turtl::create_turtl_backup_from_directory(complete_path, user_id).unwrap();
        println!("{:#}", j);
    }
}
