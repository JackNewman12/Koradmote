#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

use std::io;
use std::path::{Path, PathBuf};

use std::env;

use rocket::response::NamedFile;

#[get("/")]
fn index() -> io::Result<NamedFile> {
    println!("\nCurrent path is {:?}", env::current_dir().unwrap());
    // NamedFile::open("build/static/index.html")
    NamedFile::open("build/index.html")
}

#[get("/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("build/").join(file)).ok()
}

fn main() {
    rocket::ignite().mount("/", routes![index, files]).launch();
}