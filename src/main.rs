use std::env;
mod evernote2turtl;


fn parse_arg(args: &[String]) -> (&str, u32, &str, bool) {
	if args.len() < 3 {
        println!("Usage: {} [zip file or unzipped directory path] [numeric user ID] [input format:evernote (default)|keep] [from: json (default)|html]\n",
				args[0]);
		println!("- To find your 'numeric user ID' generate a Turtl backup");
		println!("- Currently ZIP files are only supported for Evernote backups");
		println!("- JSON can only be parsed from Keep");
        std::process::exit(0);
	}

	let user_id = match args[2].parse::<u32>() {
		Ok(v) => v,
		Err(e) => {
			println!("Failed to parse numeric user id: {}", e);
			std::process::exit(0);
		},
	};

	let format;
    if args.len() >= 4 {
        format = args[3].as_str();
    } else {
        format = "evernote";
    }

	let use_json = if args.len() >= 5 && args[4].as_str() != "json" || format == "evernote" {
		false
	} else {
		true
	};

	(args[1].as_str(), user_id, format, use_json)
}

fn main() {
    let args: Vec<String> = env::args().collect();
	let (complete_path, user_id, format, use_json) = parse_arg(&args);
	println!("{}", complete_path);
	let (_path, extension) = match complete_path.rsplit_once(".") {
		None => (complete_path, ""),
		Some(t) => t,
	};
	if extension == ".zip" {
        // Zip file
        assert_eq!(format, "evernote"); // For now, no Keep ZIPs
        let j = evernote2turtl::create_turtl_backup_from_zipfile(complete_path, user_id).unwrap();
        println!("{:#}", j);
    } else {
        // unzipped directory
        let j = evernote2turtl::create_turtl_backup_from_directory(complete_path, user_id, format, use_json).unwrap();
        println!("{:#}", j);
    }
}
