This implements a converter from Google Takeout exported Keep files (zipped or unzipped json files) into [turtl](https://github.com/turtl) backup format, which can be imported into turtl.

# Setup
* [Install Rust and Cargo](https://www.rust-lang.org/tools/install)
* Checkout this repo
* Run `cargo build --release` to build the release binary

# Run
To convert a zipped takeout file into a Turtl backup file:
```
./target/release/evernote2turtl <zip_file or folder> USER_ID
```
You can get the `USER_ID` by generating a backup file for your turtl setup, and search for ```"user_id":``` at the backup file.

# [License (GNU GPL3)](LICENSE)
```
    Google Keep to Turtl
    Copyright (C) 2022 Panikon and Magnus Manske

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.
```

# Known issues
Currently when converting from zipped archives attached files (including images) are not being parsed correctly, this is due to ```convert_keep2turtl_parsefile``` trying to open the file directly from the file system and not from the ```ZipArchive```.