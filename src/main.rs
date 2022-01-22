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
mod keep2turtl;


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
	if extension == "zip" {
        // Zip file
        let j = keep2turtl::create_turtl_backup_from_zipfile(complete_path, user_id).unwrap();
        println!("{:#}", j);
    } else {
        // unzipped directory
        let j = keep2turtl::create_turtl_backup_from_directory(complete_path, user_id).unwrap();
        println!("{:#}", j);
    }
}
