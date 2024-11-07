use std::env;
use walkdir::WalkDir;

fn main() {
    let path = env::current_dir().unwrap();
    for entry in WalkDir::new(path) {
        println!("{}", entry.unwrap().path().display());
    }
}
