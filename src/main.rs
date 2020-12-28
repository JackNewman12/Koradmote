#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use serde::Serialize;

use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

#[derive(Serialize, Clone)]
struct Device {
    name: String,
    state: bool,
}

#[get("/")]
fn devices() -> Json<Vec<Device>> {
    Json([Device{name:"One".to_string(), state:true}, 
          Device{name:"Two".to_string(), state:false}].to_vec())
}

#[get("/<name>")]
fn device(name: String) -> Json<Device> {
    Json(Device{name:name, state:true})
}

#[get("/<name>/toggle/<state>")]
fn toggledevice(name: String, state: bool) -> Json<Device> {
    Json(Device{name:name, state:state})
}

fn main() {
    rocket::ignite()
    .mount("/device", routes![devices, device, toggledevice])
    .mount("/", StaticFiles::from("build/"))
    .launch();
}