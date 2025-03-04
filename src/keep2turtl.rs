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

extern crate json;
extern crate regex;
extern crate uuid;
extern crate zip;

use json::object;
use json::array;

use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use uuid::Uuid;
use zip::read::ZipArchive;

use std::io::{Error, ErrorKind};

fn get_uuid() -> String {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"-").unwrap();
    };
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let mut ret = id1.to_string();
    ret += id2.to_string().as_str();
    ret += id3.to_string().as_str();
    ret = RE.replace_all(ret.as_str(), "").to_string();
    ret.truncate(80);
    ret
}

/// Parses attachments from Keep's JSON and converts to Turtl's format
/// # Keep
/// ```
/// "attachments":[{"filePath":FILE NAME,"mimetype":FILE TYPE e.g image/jpeg}, ...],
/// ```
/// # Turtl:
/// ```
/// "file": {
/// 	"body": ,
/// 	"id": File id, can be the same as the note id,
/// 	"meta": { -> Only displayed images have this field
/// 		"height": H,
/// 		"width": W
/// 	},
/// 	"name": FILE NAME,
/// 	"size": FILE SIZE (U64),
/// 	"type": MIMETYPE
/// },
/// ```
pub fn convert_keep2turtl_parsefile(
	backup_obj: &mut json::JsonValue,
    current_dir: &str,
    keep_json: &json::JsonValue,
    j: &mut json::JsonValue,
	is_image: bool
) -> Result<bool, std::io::Error>{
	extern crate image;  // image_dimensions
	extern crate base64; // file data is saved encoded as base64

	let mut filepath = format!("{}{}{}",
		current_dir, std::path::MAIN_SEPARATOR, keep_json["attachments"][0]["filePath"]);
	let changed_extension;
	let mut file = match File::open(filepath.as_str()) {
		Ok(d) => {
			changed_extension = false;
			d
		},
		Err(_e) => {
			let temp_path = filepath.clone();
			let (path, extension) = temp_path.rsplit_once(".").unwrap();
			// Google does some funny business when Keep data is taken out, some files
			// are saved as ".jpg" but their names inside the JSON files can sometimes
			// have ".jpeg" or ".png" as the extension... 
			// Force the extension to ".jpg" and retry
			filepath = format!("{}.jpg", path);
			match File::open(filepath.as_str()) {
				Ok(d) => {
					changed_extension = true;
					d
				},
				Err(error) => {
					return Err(Error::new(ErrorKind::Other,
								format!("Invalid attachment: \"{}\" / {} (changed extension \"{}\" to \"{}\")",
										filepath,
										error,
										extension,
										".jpg"
										)
									)
							);
				},
			}
		},
	};

	let metadata = file.metadata()?;
	if !metadata.is_file() {
		return Err(Error::new(ErrorKind::Other,
					format!("Invalid attachment \"{}\" / Not a file",
							filepath
							)
						)
				);
	}

	// Read file into memory so it can be encoded as base64
	let mut buffer = Vec::with_capacity(metadata.len() as usize);
	match file.read_to_end(&mut buffer) {
		Ok(d) => d,
		Err(error) => {
			return Err(Error::new(ErrorKind::Other,
						format!("Failed to read into memory: \"{}\" / {}",
								filepath,
								error
								)
							)
					);
		},
	};
	j["file"] = object! {
		"name" => keep_json["attachments"][0]["filePath"].as_str(),
		"size" => metadata.len(),
		"type" => match changed_extension {
			false => format!("{}", keep_json["attachments"][0]["mimetype"]),
			true => {
				// Don't forget to update to the actual extension type
				let mime = format!("{}", keep_json["attachments"][0]["mimetype"]);
				let (basic, _) = mime.rsplit_once("/").unwrap();
				format!("{}/{}", basic, "jpg")
			},
		},
		"id"   => j["id"].as_str(),
		//"body" => null
	};
	if is_image {
		// Images that are displayed have the "meta" field with the dimensions
		let (height, width) = match image::image_dimensions(&filepath) {
			Ok(d) => d,
			Err(error) => {
				return Err(Error::new(ErrorKind::Other,
							format!("Invalid image: \"{}\" / {}",
									filepath,
									error
									)
								)
						);
			},
		};
		j["file"]["meta"] = object! {
			"height" => height,
			"width"  => width,
		};
	}
	// Add file data to backup object
	// Expected format { "body": null, "data": file_data base64, "id": object_uuid }
	backup_obj["files"].push(
		object!{
			"body" => json::Null,
			"data" => base64::encode(buffer),
			"id"   => j["id"].as_str(),
		}
	).unwrap();
	Ok(true)
}

/// Converts Keep's JSON format to Turtl's
pub fn convert_keep2turtl(
	backup_obj: &mut json::JsonValue,
	current_dir: &str,
    file_name: &str,
	contents: &str,
    user_id: u32,
) -> Result<json::JsonValue, std::io::Error>{
	//let (current_dir, name) = file_name.rsplit_once(std::path::MAIN_SEPARATOR).unwrap();
	println!("{}", file_name);
	//let mut f = File::open(file_name)?;
	//let mut contents = String::new();
    //f.read_to_string(&mut contents)?;

	let keep_json;
	match json::parse(&contents) {
		Ok(temp) => keep_json = temp,
		Err(error_message) => {
			return Err(Error::new(ErrorKind::Other,
						format!("Failed to parse (\"{}\") : {}",
								 file_name, error_message
								 )
							)
					);
		},
	};

	let is_file;
	let is_image;
	let mut j = object! {
        "id"       => get_uuid(),
        "space_id" => backup_obj["spaces"][0]["id"].as_str(),
		"board_id" => if keep_json["isPinned"].as_bool() == Some(false) {
							json::Null.as_str()
					} else {
							backup_obj["boards"][0/* Pinned notes*/]["id"].as_str()
					},
        "user_id"  => user_id,
		// has_file is false even when there are attachments or the type of note is file
		"has_file" => false,
		// "labels":[{"name":"label01"},{"name":"label02"}] -> "tags"["label01","label02"]
		"tags" => match keep_json["labels"].len() > 0 {
			true => {
				let mut tag_array = json::JsonValue::Array(Vec::with_capacity(keep_json["labels"].len()));
				for entry in keep_json["labels"].members() {
					match tag_array.push(entry["name"].as_str()) {
						Ok(p) => p,
						Err(e) => {
							return Err(Error::new(ErrorKind::Other,
								format!("Failed to parse (\"{}\") : {} / Failed to push labels",
										file_name,
										e
										)
									)
							);
						}
					}
					//println!("{}", entry["name"]);
				}
				tag_array
			},
			false => array![]
		},
		"title" => keep_json["title"].as_str(),
		"text"  => keep_json["textContent"].as_str(),
		// userEditedTimeStampUsec is in microsec, convert to Epoch in seconds that Turtl uses
		"mod"   => {
			let number = match keep_json["userEditedTimestampUsec"].as_u64() {
				Some(n) => n,
				None => 0,
			};
			number/10000000
		},
		// 'type' can be several different values depending on the note content
		//  text	    -> Pure text note
		//  file        -> Has an attachment (if it's an image it doesn't show in turtl)
		//  image       -> Image attachments (in Keep all notes containing images show their content by default)
		//  link        -> Bookmark notes with links (no Keep equivalent)
		//  password    -> Notes containing password (no Keep equivalent)
		"type" => match keep_json["attachments"].len() > 0 {
			true => {
				is_file = true;
				if keep_json["attachments"].len() > 1 {
					// Turtl doesn't support multiple attachments
					return Err(Error::new(ErrorKind::Other,
								format!("Failed to parse ( {} ) : {}",
										file_name,
										"Has multiple attachments, aborted conversion!"
										)
									)
							);
				}
				let mime = format!("{}", keep_json["attachments"][0]["mimetype"]);
				let token:Vec<&str>= mime.split("/").collect();
				if token.len() != 2 {
					return Err(Error::new(ErrorKind::Other,
								format!("Failed to parse ( {} ) : {} '{}'",
										file_name,
										"Has invalid attachment mimetype",
										keep_json["attachments"][0]["mimetype"]
										)
									)
							);
				}
				if token[0] == "image" {
					is_image = true;
					"image"
				} else {
					is_image = false;
					"file"
				}
			},
			false => {is_file = false; is_image = false; "text"},
		},
	};

	if is_file {
		match convert_keep2turtl_parsefile(backup_obj, &current_dir, &keep_json, &mut j, is_image) {
			Ok(r) => r,
			Err(error) => {
				return Err(Error::new(ErrorKind::Other,
							format!("Failed to parse(\"{}\"): {}",
									file_name,
									error
									)
								)
						);
			},
		};
	}

	if keep_json["listContent"].len() > 0 {
		// Turtl doesn't support lists directly, only by markdown
		// "listContent":[{"text": , "isChecked": bool}, ...]
		// - [ ] Unmarked item
		// - [x] Marked item
		let mut list_vec = Vec::with_capacity(keep_json["listContent"].len() as usize);
		for entry in keep_json["listContent"].members() {
			list_vec.push(format!(
									"- [{}] {}\n",
									if entry["isChecked"] == true { "X" } else { " " },
									entry["text"]
								)
						);
			
		}
		j["text"] = json::JsonValue::String(list_vec.concat());
		//println!("{}", j["text"]);
	}

	Ok(j)
}

/// Generates basic backup object
pub fn get_turtl_backup_object(
    user_id: u32
) -> Result<json::JsonValue, std::io::Error> {
    let space_name;
    let space_color;
	let space_id = get_uuid().to_string();

	space_name = "Google Keep import";
	space_color = "#d88742";

    let ret = object! {
        "body" => json::Null ,
        "boards" => array![
			object!{
				"body"    => json::Null,
				"id"      => get_uuid().to_string(),
				"user_id" => user_id,
				"keys"    => array![],
				"space_id"=> space_id.clone(),
				"title"   => "Pinned notes"
			}
		],
        "files" => array![],
        "notes" => array![],
        "schema_version" => 2 ,
        "spaces" => array![
            object!{
                "color" => space_color,
                "id" => space_id,
                "user_id" => user_id,
                "invites" => array![],
                "keys" => array![],
                "members" => array![],
                "title" => space_name,
            }
        ]
    };
    Ok(ret)
}

/// Creates backup from a valid takeout zip file
/// Expected takeout zip structure:
/// ```Takeout/Keep```
pub fn create_turtl_backup_from_zipfile(
    zipfile: &str,
    user_id: u32,
) -> Result<json::JsonValue, std::io::Error> {
	lazy_static::lazy_static! {
		static ref RE_VALID_FILE: Regex = Regex::new("^Takeout/Keep/(.+)\\.json$").unwrap();
	};
	let mut ret = get_turtl_backup_object(user_id)?;

    let f = File::open(zipfile)?;
    let mut zip = ZipArchive::new(f)?;
	for i in 0..zip.len() {
		let mut file = zip.by_index(i).unwrap();

		if RE_VALID_FILE.is_match(file.name()) {
			println!("{}", file.name());

			let mut contents = String::new();
			file.read_to_string(&mut contents)?;
			match convert_keep2turtl(&mut ret, "Takeout/Keep", file.name(), &contents, user_id) {
				Ok(o) => {
					ret["notes"].push(o).unwrap();
					continue
				},
				Err(e) => println!("\n{}(Takeout/Keep): {}", zipfile, e),
			}
		}
	}
	Ok(ret)
}

/// Creates a backup from a directory containing Keep .json files
pub fn create_turtl_backup_from_directory(
    path: &str,
    user_id: u32,
) -> Result<json::JsonValue, std::io::Error> {
    let mut ret = get_turtl_backup_object(user_id)?;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path().to_string_lossy().into_owned();
				if file_path.len() < 4 { continue; }
				let extension = &file_path[file_path.len()-4..];

				if extension == "json" {
					let mut f = File::open(file_path.as_str())?;
					let mut contents = String::new();
					f.read_to_string(&mut contents)?;

					match convert_keep2turtl(&mut ret, path, file_path.as_str(), &contents, user_id) {
						Ok(o) => {
							ret["notes"].push(o).unwrap();
							continue
						},
						Err(e) => println!("\n{}: {}", file_path.as_str(), e),
					}
				}
            }
        }
    } else {
		println!("Failed to open target path {} (working dir {})\n",
		path,
		std::env::current_dir()?.display());
	}
    Ok(ret)
}
