#![feature(proc_macro_hygiene, decl_macro)]
#![feature(async_closure)]
#[macro_use]
extern crate rocket;

use clap::Clap;
use futures::{executor::block_on, future::join_all};

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rocket::State;
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

struct Device {
    connection: ka3005p::Ka3005p,
    state: DeviceState,
}

impl Device {
    async fn update_state(&mut self) {
        let res = self.connection.status();
        println!("wow");
        match res {
            Ok(status) => {
                // println!("{}", Json(status).to_string());
                self.state.voltage = status.voltage;
                self.state.current = status.current;
                self.state.power = status.flags.output.into();
            }
            Err(err) => println!("Update PSU failed! {}", err),
        }
    }
    fn set_power(&mut self, output: bool) {
        // Do what the user asked
        println!("Setting power to {:?}", output);
        self.connection.execute(ka3005p::Command::Power(output.into()))
        .expect("Sending Command Failed! {}");
        // Update state to reflect all changes
        block_on(self.update_state());
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
}

#[get("/<name>/toggle")]
fn toggledevice(name: String, devs: State<DeviceList>) -> Option<Json<DeviceState>> {
    let mut devlock = devs.lock().unwrap();
    let dev = devlock.get_mut(&name)?;
    // Make sure the state is up to date before we attempt to do the logic not
    block_on(dev.update_state());
    dev.set_power(!dev.state.power);
    Some(Json(dev.state))
}

#[get("/<name>/toggle/<state>")]
fn setdevice(name: String, state: bool, devs: State<DeviceList>) -> Option<Json<DeviceState>> {
    let mut devlock = devs.lock().unwrap();
    let dev = devlock.get_mut(&name)?;
    dev.set_power(state);
    Some(Json(dev.state))
}

fn update_device_states(devs: DeviceList) {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("Start");
        let mut uhh = devs.lock().unwrap();
        let updateiterator: Vec<_> = uhh.values_mut().map(|d| Box::pin(d.update_state())).collect();
        block_on(join_all(updateiterator));
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
            let port = match ka3005p::Ka3005p::new(&chunk[1])
            {
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
