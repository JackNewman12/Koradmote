#![feature(decl_macro)]

use clap::Clap;

use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rust_embed::RustEmbed;
use tokio;
use warp::{reject::Reject, Filter};

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

async fn devices(devs: DeviceList) -> Result<warp::reply::Json, warp::Rejection> {
    let data = devs.lock().unwrap();
    Ok(warp::reply::json(
        &data
            .iter()
            .map(|(k, d)| (k.clone(), d.state))
            .collect::<BTreeMap<String, DeviceState>>(),
    ))
}

async fn device(name: String, devs: DeviceList) -> Result<warp::reply::Json, warp::Rejection> {
    let data = devs.lock().unwrap();
    match data.get(&name) {
        Some(data) => Ok(warp::reply::json(&data.state)),
        _ => Err(warp::reject::not_found()),
    }
}

#[derive(Debug)]
struct ToggleFailed;
impl Reject for ToggleFailed {}
async fn toggledevice(
    name: String,
    devs: DeviceList,
) -> Result<warp::reply::Json, warp::Rejection> {
    let mut devlock = devs.lock().unwrap();
    let dev = devlock
        .get_mut(&name)
        .ok_or_else(|| warp::reject::not_found())?;
    // Given the user interface is blocked while using serial, we can assume the state is the same as the last update
    dev.set_power(!dev.state.power)
        .map_err(|_| warp::reject::custom(ToggleFailed))?;
    Ok(warp::reply::json(&dev.state))
}

async fn setdevice(
    name: String,
    state: bool,
    devs: DeviceList,
) -> Result<warp::reply::Json, warp::Rejection> {
    let mut devlock = devs.lock().unwrap();
    let dev = devlock
        .get_mut(&name)
        .ok_or_else(|| warp::reject::not_found())?;
    dev.set_power(state)
        .map_err(|_| warp::reject::custom(ToggleFailed))?;
    Ok(warp::reply::json(&dev.state))
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

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    // println!("{:?}", opts);
    if opts.power_supplies.len() % 2 != 0 {
        eprintln!("Input devices must be groups of two!");
        return;
    }

    // Create the devices struct
    let current_devices = DeviceList::default();

    // Pass it to the updater thread
    {
        let dev_arc = current_devices.clone();
        std::thread::spawn(move || update_device_states(dev_arc));
    }

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

    //
    let route = warp::any().and(warp_embed::embed(&WebAssets)); //.with(warp::compression::gzip());

    let devices_filter = warp::any().map(move || current_devices.clone());
    let alldevices = warp::path!("device")
        .and(devices_filter.clone())
        .and_then(devices);
    let singledevice = warp::path!("device" / String)
        .and(devices_filter.clone())
        .and_then(device);
    let toggledevice = warp::path!("device" / String / "toggle")
        .and(devices_filter.clone())
        .and_then(toggledevice);
    let setdevice = warp::path!("device" / String / "toggle" / bool)
        .and(devices_filter.clone())
        .and_then(setdevice);

    let routes = route
        .or(alldevices)
        .or(singledevice)
        .or(toggledevice)
        .or(setdevice);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
