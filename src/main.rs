#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::serve::StaticFiles;

#[get("/")]
fn devices() -> &'static str {
    "[{'wow':3}, {'swag':4}]"
}

#[get("/<name>")]
fn device(name: String) -> String {
    format!("{{{}:3}}", name)
}

#[get("/<name>/toggle/<state>")]
fn toggledevice(name: String, state: bool) -> String {
    format!("{{{}:{}}}", name, state)
}

fn main() {
    rocket::ignite()
    .mount("/device", routes![devices, device, toggledevice])
    .mount("/", StaticFiles::from("build/"))
    .launch();
}