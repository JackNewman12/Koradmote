# A remote for Korad power supplies

A simple Korad (and clones) power supply server.

![Screenshot](https://github.com/JackNewman12/PSUReact/blob/main/Screenshot.png)


Design goals:
 * Dirt simple REST API so other automated frameworks can toggle devices
 * No changing of power / current settings. i.e. Idiot-proof
 * Simple frontend and backend design - single source file design
 * Single binary output - all files embedded

## Running
Grab the binaries from the Releases.
```
./koradmote Dev1 /dev/tty/Blah Dev2 /dev/tty/Junk
```

## API
### All Devices
`/device/` Returns the current state of all devices
```
curl -s localhost:8000/device/
{
  "Dev1" :{"voltage":0.0, "current":0.0, "power":false},
  "Dev2" :{"voltage":0.0, "current":0.0, "power":false},
  "Dev3" :{"voltage":0.0, "current":0.0, "power":false},
  "Etc🚀":{"voltage":0.0, "current":0.0, "power":false}
}
```
### Single Device
`/device/{name}/` Returns the state of just a single device
```
curl -s localhost:8000/device/Dev1
  {"voltage":0.0, "current":0.0, "power":false}
```
### Toggle Devices
`/device/{name}/toggle` will toggle the device
`/device/{name}/toggle/true``toggle/false` will set the device to the target state
The reponse will return the state of the power supply after the toggle.
```
curl -s localhost:8000/device/Dev1/toggle
  {"voltage":0.0, "current":0.0, "power":false}
```

```
curl -s localhost:8000/device/Dev1/toggle/false
  {"voltage":0.0, "current":0.0, "power":false}
```

# Building
Assuming Node.js and Rust are installed
{{{
  npm install
  cargo build --release
}}}

Note that the debug version of the binary will use the files in the `/build/` folder for faster development. The release build will embed the files inside the binary