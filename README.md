[![Build Status](https://travis-ci.org/tsathishkumar/MySController-rs.svg?branch=master)](https://travis-ci.org/tsathishkumar/MySController-rs) [ ![Download](https://api.bintray.com/packages/tsathishkumar/MySController-rs/mys-controller-rs/images/download.svg) ](https://bintray.com/tsathishkumar/MySController-rs/mys-controller-rs/_latestVersion)
# MySController-rs
Proxy controller for MySensors written in Rust lang. It is to perform OTA firmware updates, and proxy all other requests to the actual controllers like homeassist. Mainly to add OTA support for homeassist controller, but can work with any other controllers.

This server acts as a proxy between Gateway and the Controller. Both might be either connected through a serial port or a TCP connection.

Before running the server, set the correct connection type and connection port for Gateway and Controller in conf.ini file.

To run the proxy server:
```
cargo run
```

Note: If you are using TCP for controller - the port value will be used to create TCP server listening on the specified port. (So it shoud be the address of the machine running MyRController)

## TODO

- [x] Gracefully handle connection at both side, i.e never panic and wait for both connections
- [ ] Ability to handle ota requests even when there is no controller connected
- [ ] Manage firmwares type and version, ability to upload newer versions of firmwares, expose apis
- [ ] Manage nodes and the firmwares installed, expose api's
- [ ] Add an endpoint to assign a firmware to particular node
- [ ] Manage requested firmware for nodes
