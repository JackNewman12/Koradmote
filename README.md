# A remote for Korad power supplies

A simple Korad (and clones) power supply server.

![Screenshot](https://github.com/JackNewman12/PSUReact/blob/main/Screenshot.png)


**Design Goals:**
 * Dirt simple REST API so other automated frameworks can toggle devices
 * No changing of voltage / current. i.e. Idiot-proof
 * Simple frontend and backend design - only a single source
 * Single binary output - all files embedded, no mucking around

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
`/device/{name}/` Returns the state of just a single device or `404 Not Found` for invalid device name
```
curl -s localhost:8000/device/Dev1
  {"voltage":0.0, "current":0.0, "power":false}
```
### Toggle Devices
`/device/{name}/toggle` will toggle the device

`/device/{name}/toggle/true` will set the device to the target state

The reponse will return the state of the power supply after the toggle or `404 Not Found` for invalid device name and `500 Internal Server Error` for toggle/serial comms failure.
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
```
  npm install
  npm run build
  cargo build --release
```

Note that the debug version of the binary will use the files in the `/build/` folder for faster development. The release build will embed the files inside the binary

You can create some virtual serial ports on linux via:
```
sudo socat -d -d pty,link=/dev/ttyS0,raw,echo=0 pty,link=/dev/ttyS1,raw,echo=0
```

# Logging
There are various logging levels implemented for this application:
```
LOG=DEBUG ./koradmote
LOG=INFO ./koradmote
LOG=OFF ./koradmote
```
