#[macro_use]
extern crate json;
extern crate regex;

use regex::Regex;
use std::env;
mod evernote2turtl;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 && args.len() != 4 && args.len() != 5 {
        println!("Usage: {} [zip file or unzipped directory path] [numeric user ID] [input format:evernote (default)|keep] [from: json (default)|html]\n",
				args[0]);
		println!("- To find your 'numeric user ID' generate a Turtl backup");
		println!("- Currently ZIP files are only supported for Evernote backups");
		println!("- JSON can only be parsed from Keep");
        return;
    }
    let format;
    if args.len() >= 4 {
        format = args[3].as_str();
    } else {
        format = "evernote";
    }
    let path = args[1].as_str();

	let user_id;
	match args[2].parse::<u32>() {
		Ok(integer) => user_id = integer,
		Err(user_id) => {
			println!("Failed to parse numeric user id: {}", user_id);
			return;
		},
	};

	let use_json = if args[4].as_str() != "json" || format == "evernote" {
		false
	} else {
		true
	};

    let re: Regex = Regex::new(r"\.zip$").unwrap();
    if re.is_match(path) {
        // Zip file
        assert_eq!(format, "evernote"); // For now, no Keep ZIPs
        let j = evernote2turtl::create_turtl_backup_from_zipfile(path, user_id).unwrap();
        println!("{:#}", j);
    } else {
        // unzipped directory
        let j = evernote2turtl::create_turtl_backup_from_directory(path, user_id, format, use_json).unwrap();
        println!("{:#}", j);
    }
}
