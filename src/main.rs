#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;

use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, println};

use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use serde::Serialize;
use serialport::SerialPort;

use rust_embed::RustEmbed;
use rust_embed_rocket;

#[derive(RustEmbed)]
#[folder = "build/"]
struct WebAssets;

#[derive(Serialize, Clone, Copy, Default, Debug)]
struct DeviceState {
    voltage: u32,
    current: u32,
    power: bool,
}

struct Device {
    connection: Box<dyn SerialPort>,
    state: DeviceState,
}

impl Device {
    fn update_state(&mut self) {
        let mut buf: [u8; 10] = [0; 10];
        self.connection.write(b"SomeStuff").expect("Wrote to PSU");
        // self.connection.read(&mut buf).expect("PSU returned data");
        // TODO - Decode bytes and update state
    }
    fn set_power(&mut self, output: bool) {
        // Do what the user asked
        // println!("Setting power to {:?}", output);
        self.connection.write(b"SomeStuff").expect("Wrote to PSU");
        // Update state to reflect all changes
        self.update_state();
    }
}

type DeviceList = Arc<Mutex<BTreeMap<String, Device>>>;

#[get("/")]
fn devices(devs: State<DeviceList>) -> Json<BTreeMap<String, DeviceState>> {
    let data = devs.lock().unwrap();
    Json(data.iter().map(|(k, d)| (k.clone(), d.state)).collect())
}

#[get("/<name>")]
fn device(name: String, devs: State<DeviceList>) -> Option<Json<DeviceState>> {
    let data = devs.lock().unwrap();
    Some(Json(data.get(&name)?.state))
    // Json(Device{name:name, state:true})
}

#[get("/<name>/toggle")]
fn toggledevice(name: String, devs: State<DeviceList>) -> Option<Json<DeviceState>> {
    let mut devlock = devs.lock().unwrap();
    let mut dev = devlock.get_mut(&name)?;
    // Make sure the state is up to date before we attempt to do the logic not
    dev.update_state();
    dev.set_power(!dev.state.power);
    dev.state.power = !dev.state.power; // FIXME just do this to simulate device
    Some(Json(dev.state))
}

#[get("/<name>/toggle/<state>")]
fn setdevice(name: String, state: bool, devs: State<DeviceList>) -> Option<Json<DeviceState>> {
    let mut devlock = devs.lock().unwrap();
    let mut dev = devlock.get_mut(&name)?;
    dev.set_power(state);
    dev.state.power = state; // FIXME just do this to simulate device
    Some(Json(dev.state))
}

fn update_device_states(devs: DeviceList) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        for (_, d) in devs.lock().unwrap().iter_mut() {
            d.update_state();
        }
    }
}

fn main() {
    let rocket = rocket::ignite()
        .mount("/device", routes![devices, device, toggledevice, setdevice])
        // .mount("/", StaticFiles::from("build/"))
        .mount(
            "/",
            rust_embed_rocket::Server::from_config(
                WebAssets,
                rust_embed_rocket::Config {
                    serve_index: true,
                    ..Default::default()
                },
            ),
        )
        .manage(DeviceList::default());

    let current_devices = rocket.state::<DeviceList>().unwrap();
    {
        let mut devlist = current_devices.lock().unwrap();
        devlist.insert(
            "Zebra".to_string(),
            Device {
                connection: serialport::new("/dev/pts/2", 115200).open().unwrap(),
                state: Default::default(),
            },
        );
        devlist.insert(
            "Alp".to_string(),
            Device {
                connection: serialport::new("/dev/pts/2", 115200).open().unwrap(),
                state: Default::default(),
            },
        );
    }

    let dev_arc = current_devices.clone();
    std::thread::spawn(move || update_device_states(dev_arc));
    rocket.launch();

    println!("End!");
}
