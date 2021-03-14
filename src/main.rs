#![feature(decl_macro)]
#![deny(
    clippy::all,
    clippy::pedantic,
)]
use clap::Clap;

use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rust_embed::RustEmbed;
use warp::{reject::Reject, Filter};

#[macro_use] extern crate log;

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
        debug!("Updating State: {:?}", self.state);
        Ok(self.state)
    }

    fn set_power(&mut self, output: bool) -> anyhow::Result<DeviceState> {
        // Do what the user asked
        info!("Setting power to {:?}", output);
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
        None => Err(warp::reject::not_found()),
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
    let dev = devlock.get_mut(&name).ok_or_else(warp::reject::not_found)?;
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
    let dev = devlock.get_mut(&name).ok_or_else(warp::reject::not_found)?;
    dev.set_power(state)
        .map_err(|_| warp::reject::custom(ToggleFailed))?;
    Ok(warp::reply::json(&dev.state))
}

/// Update each of the device
fn update_device_states(devs: &DeviceList) {
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
                        Err(e) => error!("Device '{}' - {}", k, e),
                    };
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

/// Print a list of ports that are probably power supplies
fn find_devices() {
    let allports= serialport::available_ports()
    .expect("Could not search for serialports!");
    debug!("{:?}", allports);

    let ports:Vec<serialport::SerialPortInfo> = allports.into_iter()
    .filter(|info| match &info.port_type {
        serialport::SerialPortType::UsbPort(usb) => usb.vid == 1046,
        _ => false,
    })
    .collect();

    println!("Devices that are most likely PSUs:");
    for port in ports {
        println!("{:?}", port);
    }
}

#[derive(Clap, Debug)]
struct Opts {
    /// List of power supples "Name" "Port" "Name" "Port"
    #[clap()]
    power_supplies: Vec<String>,
}

#[tokio::main]
async fn main() {
    // Setup logging
    if std::env::var("LOG").is_err() {
        std::env::set_var("LOG", "INFO");
    }
    pretty_env_logger::init_custom_env("LOG");

    // Opts parsing
    let opts: Opts = Opts::parse();
    debug!("{:?}", opts);

    // Print any devices we can find for the user
    if opts.power_supplies.is_empty() {
        find_devices();
        return;
    }

    if opts.power_supplies.len() % 2 != 0 {
        eprintln!("Input devices must be groups of two!");
        return;
    }

    // Create the devices struct
    let current_devices = DeviceList::default();

    // Pass it to the updater thread
    {
        let dev_arc = current_devices.clone();
        std::thread::spawn(move || update_device_states(&dev_arc));
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
            debug!("Created Device: {:?}", chunk[0]);
            devlist.insert(
                chunk[0].to_string(),
                Device {
                    connection: port,
                    state: DeviceState::default(),
                },
            );
        }

    }

    // Create base route
    let route = warp::any()
        .and(warp_embed::embed(&WebAssets))
        .with(warp::compression::gzip());

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

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}
