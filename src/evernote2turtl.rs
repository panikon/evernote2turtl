extern crate regex;
extern crate uuid;
extern crate zip;
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use uuid::Uuid;
use zip::read::ZipArchive;

fn get_dummy_turtl_space_id() -> String {
    lazy_static::lazy_static! {
        static ref SPACE_ID: String = get_uuid();
    }
    SPACE_ID.to_string()
}

fn extract_title(html: &str) -> Result<String, std::io::Error> {
    let mut ret = String::new();
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"<title>\s*(.+?)\s*</title>").unwrap();
    }
    for cap in RE.captures_iter(html) {
        ret = cap[1].to_string();
        break;
    }
    Ok(ret)
}

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

pub fn convert_html_file_contents_to_json(
    contents: std::string::String,
    user_id: u32,
    format: &str,
) -> Result<json::JsonValue, std::io::Error> {
    let html_start = contents.find("<body").unwrap();
    let html = contents.get(html_start..).unwrap();
    let title = extract_title(&contents)?;
    let md = html2md::parse_html(&html);
    let mut md2 = md.trim_matches(|c| c == ' ' || c == '\n');
    if format == "keep" && md2.contains("\n") {
        // Remove first line in Keep format, as it just repeats the title
        let second_line = md2.find("\n").unwrap();
        md2 = md2.get((second_line + 1)..).unwrap();
        md2 = md2.trim_matches(|c| c == ' ' || c == '\n');
    }
    let j = object! {
        "id" => get_uuid(),
        "space_id" => get_dummy_turtl_space_id() ,
        "user_id" => user_id ,
        "has_file" => false ,
        "tags" => array![] ,
        "title" => title ,
        "text" => md2 ,
        "type" => "text"
    };
    Ok(j)
}
use std::io::{Error, ErrorKind};


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
		"name" => filepath.as_str(),
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
		"data" => base64::encode(buffer),
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
	Ok(true)
}

/// Converts Keep's JSON format to Turtl's
pub fn convert_keep2turtl(
	current_dir: &str,
    file_name: &str,
    user_id: u32,
) -> Result<json::JsonValue, std::io::Error>{
	//let (current_dir, name) = file_name.rsplit_once(std::path::MAIN_SEPARATOR).unwrap();
	println!("{}", file_name);
    let mut f = File::open(file_name)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

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
        "space_id" => get_dummy_turtl_space_id(),
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
		match convert_keep2turtl_parsefile(&current_dir, &keep_json, &mut j, is_image) {
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

pub fn convert_file_to_json(
    file_name: &str,
    user_id: u32,
    format: &str,
) -> Result<json::JsonValue, std::io::Error> {
    let mut f = File::open(file_name)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    convert_html_file_contents_to_json(contents, user_id, format)
}

pub fn get_turtl_backup_object(
    user_id: u32,
    format: &str,
) -> Result<json::JsonValue, std::io::Error> {
    let space_name;
    let space_color;
    if format == "evernote" {
        space_name = "Evernote import";
        space_color = "#000000";
    } else {
        space_name = "Google Keep import";
        space_color = "#0000BB";
    }
    let ret = object! {
        "body" => json::Null ,
        "boards" => array![],
        "files" => array![],
        "notes" => array![],
        "schema_version" => 2 ,
        "spaces" => array![
            object!{
                "color" => space_color,
                "id" => get_dummy_turtl_space_id(),
                "user_id" => user_id ,
                "invites" => array![],
                "keys" => array![],
                "members" => array![],
                "title" => space_name,
            }
        ]
    };
    Ok(ret)
}

pub fn create_turtl_backup_from_zipfile(
    zipfile: &str,
    user_id: u32,
) -> Result<json::JsonValue, std::io::Error> {
    lazy_static::lazy_static! {
        static ref RE_HTML: Regex = Regex::new(r"\.html$").unwrap();
        static ref RE_HIDDEN1: Regex = Regex::new(r"/\.").unwrap();
        static ref RE_HIDDEN2: Regex = Regex::new(r"^\.").unwrap();
    };
    let mut ret = get_turtl_backup_object(user_id, "Evernote import")?;
    let f = File::open(zipfile)?;
    let mut zip = ZipArchive::new(f)?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).unwrap();
        let filename = file.name();
        if RE_HTML.is_match(filename)
            && !RE_HIDDEN1.is_match(filename)
            && !RE_HIDDEN2.is_match(filename)
            && file.size() > 0
        {
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let note = convert_html_file_contents_to_json(contents, user_id, "evernote")?; // Using evernote constant until this can process Keep ZIP files as well
            ret["notes"].push(note).unwrap();
        }
    }
    Ok(ret)
}

pub fn create_turtl_backup_from_directory(
    path: &str,
    user_id: u32,
    format: &str,
	use_json: bool,
) -> Result<json::JsonValue, std::io::Error> {
    lazy_static::lazy_static! {
        static ref RE_HTML: Regex = Regex::new(r"\.html$").unwrap();
		static ref RE_JSON: Regex = Regex::new(r"\.json$").unwrap();
    };
    let mut ret = get_turtl_backup_object(user_id, format)?;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path().to_string_lossy().into_owned();
				if use_json {
					if RE_JSON.is_match(file_path.as_str()) {
						match convert_keep2turtl(path, file_path.as_str(), user_id) {
							Ok(o) => {
								ret["notes"].push(o).unwrap();
								continue
							},
							Err(e) => println!("\n{}: {}", file_path.as_str(), e),
						}
					}
				} else {
					if RE_HTML.is_match(file_path.as_str()) {
						let note = convert_file_to_json(file_path.as_str(), user_id, format)?;
						ret["notes"].push(note).unwrap();                    
					}
				}
            }
        }
    }
    Ok(ret)
}
