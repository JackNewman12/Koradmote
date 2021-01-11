#![feature(decl_macro)]
#[macro_use]
extern crate rocket;

use clap::Clap;

use anyhow;
use std::sync::{Arc, Mutex};
use std::{collections::BTreeMap, io::Cursor};

use rocket::http::Status;
use rocket::{Response, State};
use rocket_contrib::json::Json;
use serde::Serialize;

use rust_embed::RustEmbed;
use rust_embed_rocket;

#[derive(RustEmbed)]
#[folder = "build/"]
struct WebAssets;

#[derive(Serialize, Clone, Copy, Default, Debug)]
struct DeviceState {
    voltage: f32,
    current: f32,
    power: bool,
}

type DeviceList = Arc<Mutex<BTreeMap<String, Device>>>;

struct Device {
    connection: ka3005p::Ka3005p,
    state: DeviceState,
}

impl Device {
    fn update_state(&mut self) -> anyhow::Result<DeviceState> {
        let status = self.connection.status()?;
        self.state.voltage = status.voltage;
        self.state.current = status.current;
        self.state.power = status.flags.output.into();
        Ok(self.state)
    }

    fn set_power(&mut self, output: bool) -> anyhow::Result<DeviceState> {
        // Do what the user asked
        println!("Setting power to {:?}", output);
        self.connection
            .execute(ka3005p::Command::Power(output.into()))?;
        // Update state to reflect all changes
        self.update_state()
    }
}

#[get("/")]
fn devices(devs: State<DeviceList>) -> Json<BTreeMap<String, DeviceState>> {
    let data = devs.lock().unwrap();
    Json(data.iter().map(|(k, d)| (k.clone(), d.state)).collect())
}

#[get("/<name>")]
fn device(name: String, devs: State<DeviceList>) -> Option<Json<DeviceState>> {
    let data = devs.lock().unwrap();
    Some(Json(data.get(&name)?.state))
}

#[get("/<name>/toggle")]
fn toggledevice(name: String, devs: State<DeviceList>) -> Result<Json<DeviceState>, Response> {
    let mut devlock = devs.lock().unwrap();
    let dev = devlock.get_mut(&name).ok_or_else(|| {
        Response::build()
            .status(Status::NotFound)
            .sized_body(Cursor::new("Device not found"))
            .finalize()
    })?;
    // Given the user interface is blocked while using serial, we can assume the state is the same as the last update
    dev.set_power(!dev.state.power).map_err(|_| {
        Response::build()
            .status(Status::InternalServerError)
            .sized_body(Cursor::new("Failed to toggle device"))
            .finalize()
    })?;
    Ok(Json(dev.state))
}

#[get("/<name>/toggle/<state>")]
fn setdevice(
    name: String,
    state: bool,
    devs: State<DeviceList>,
) -> Result<Json<DeviceState>, Response> {
    let mut devlock = devs.lock().unwrap();
    let dev = devlock.get_mut(&name).ok_or_else(|| {
        Response::build()
            .status(Status::NotFound)
            .sized_body(Cursor::new("Device not found"))
            .finalize()
    })?;
    dev.set_power(state).map_err(|_| {
        Response::build()
            .status(Status::InternalServerError)
            .sized_body(Cursor::new("Failed to toggle device"))
            .finalize()
    })?;
    Ok(Json(dev.state))
}

/// Update each of the device
fn update_device_states(devs: DeviceList) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let mut devices = devs.lock().unwrap();
        let device_replace = std::mem::replace(&mut *devices, BTreeMap::new());
        let update_threads: Vec<_> = device_replace
            .into_iter()
            .map(|(k, mut d)| {
                std::thread::spawn(move || {
                    match d.update_state() {
                        Ok(_) => {}
                        Err(e) => println!("Update Failed - {} - {}", k, e),
                    }
                    (k, d)
                })
            })
            .collect();

        *devices = update_threads
            .into_iter()
            .map(|t| t.join().unwrap())
            .collect();
    }
}

#[derive(Clap, Debug)]
struct Opts {
    /// List of power supples "Name" "Port" "Name" "Port"
    #[clap(required(true))]
    power_supplies: Vec<String>,

    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

fn main() {
    let opts: Opts = Opts::parse();

    // println!("{:?}", opts);
    if opts.power_supplies.len() % 2 != 0 {
        eprintln!("Input devices must be groups of two!");
        return;
    }

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
        for chunk in opts.power_supplies.chunks_exact(2) {
            let port = match ka3005p::Ka3005p::new(&chunk[1]) {
                Ok(port) => port,
                Err(e) => {
                    eprintln!("Serial port failure: {}", e);
                    return;
                }
            };

            devlist.insert(
                chunk[0].to_string(),
                Device {
                    connection: port,
                    state: Default::default(),
                },
            );
        }
    }

    let dev_arc = current_devices.clone();
    std::thread::spawn(move || update_device_states(dev_arc));

    // Start the rocket server. This blocks forever
    rocket.launch();
}
