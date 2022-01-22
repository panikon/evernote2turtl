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

