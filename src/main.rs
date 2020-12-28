#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use std::sync::Mutex;
use std::collections::HashMap;

use serde::Serialize;

use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

#[derive(Serialize, Clone, Default)]
struct DeviceState {
    // voltage: u32,
    // current: u32,
    power: bool,
}

#[derive(Serialize, Clone, Default)]
struct Device {
    connection: String, // TODO will be some sort of Serial device type
    state: DeviceState,
}

type DeviceList = Mutex<HashMap<String, Device>>;

#[get("/")]
fn devices(devs: State<DeviceList>) -> Json<Vec<DeviceState>> {
    let data = devs.lock().unwrap();
    Json(data.clone().into_iter().map(|(k, d)| d.state).collect())
}

#[get("/<name>")]
fn device(name: String, devs: State<DeviceList>) -> Json<Device> {
    let data = devs.lock().unwrap();
    Json(data.get(&name).unwrap().clone())
    // Json(Device{name:name, state:true})
}

#[get("/<name>/toggle/<state>")]
fn toggledevice(name: String, state: bool) -> Json<Device> {
    // Json(Device{name:name, state:state})
    Json(Default::default())
}

fn main() {
    let rocket = rocket::ignite()
    .mount("/device", routes![devices, device, toggledevice])
    .mount("/", StaticFiles::from("build/"))
    .manage(DeviceList::new(HashMap::new()));

    let current_devices = rocket.state::<DeviceList>().unwrap();
    {
    let mut devlist = current_devices.lock().unwrap();
    devlist.insert("One".to_string(), Device{..Default::default()});
    }

    rocket.launch();
}