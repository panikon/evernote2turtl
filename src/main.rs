//    Google Keep to Turtl
//    Copyright (C) 2022 Panikon and Magnus Manske
//
//    This program is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    This program is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    You should have received a copy of the GNU General Public License
//    along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::env;
use std::fs::File;
use std::io::prelude::*;
mod keep2turtl;


fn parse_arg(args: &[String]) -> (&str, u32, &str) {
	if args.len() < 3 {
        println!("Usage: {} [zip file or unzipped directory path] [numeric user ID] [output file | defaults to import.json]\n",
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

	(args[1].as_str(), user_id, 
		if args.len() > 3 {
			args[3].as_str()
		} else {
			"import.json"
		}
	)
}

fn main() {
    let args: Vec<String> = env::args().collect();
	let (complete_path, user_id, target) = parse_arg(&args);
	println!("{}", complete_path);
	let (_path, extension) = match complete_path.rsplit_once(".") {
		None => (complete_path, ""),
		Some(t) => t,
	};
	let j;
	if extension == "zip" {
        // Zip file
        j = keep2turtl::create_turtl_backup_from_zipfile(complete_path, user_id).unwrap();
    } else {
        // unzipped directory
        j = keep2turtl::create_turtl_backup_from_directory(complete_path, user_id).unwrap();
    }
	println!("\nSaving {} notes to {}", j["notes"].len(), target);
	let mut file = match File::create(target) {
		Ok(f) => f,
		Err(e) => panic!("Failed to create {}, error: {}", target, e),
	};
	match file.write_all(json::stringify(j).as_bytes()) {
		Ok(_f) => println!("Save successful"),
		Err(e) => panic!("Failed to save to {}, error: {}", target, e),
	};
}
