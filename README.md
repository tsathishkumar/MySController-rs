[![Build Status](https://travis-ci.org/tsathishkumar/MySController-rs.svg?branch=master)](https://travis-ci.org/tsathishkumar/MySController-rs) [ ![Download](https://api.bintray.com/packages/tsathishkumar/myscontroller-rs/myscontroller-rs/images/download.svg) ](https://bintray.com/tsathishkumar/myscontroller-rs/myscontroller-rs/_latestVersion)
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
- [x] Ability to handle ota requests even when there is no controller connected
- [x] Ability to restart the node using api
- [x] Manage nodes and the firmwares installed, expose api's 
    - GET `/nodes`
    - PUT `/node` `<node>`
    - POST `/reboot_node` `<node>`
- [x] Get node's firmware type and version from database and use it for ota request from node
- [ ] Handle auto update flag in node 
    - whenever there is new version for a firmware, it should automatically update all nodes which have auto update as `true` to latest version
- [ ] Change from file system to database storage for firmwares
- [ ] Manage firmwares type and version, ability to upload newer versions of firmwares, expose apis 
    - GET `/firmwares`
    - POST `/firmware/{type}/{version}` 
    - PUT `/firmware/{type}/{version}`
- [ ] Improve logging (parsed message for OTA request etc)
- [ ] MQTT integration
