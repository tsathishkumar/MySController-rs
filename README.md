[![Build Status](https://travis-ci.org/tsathishkumar/MySController-rs.svg?branch=master)](https://travis-ci.org/tsathishkumar/MySController-rs) [ ![Download](https://api.bintray.com/packages/tsathishkumar/myscontroller-rs/myscontroller-rs/images/download.svg) ](https://bintray.com/tsathishkumar/myscontroller-rs/myscontroller-rs/_latestVersion)
# MySController-rs
Started as a project to support OTA for MySensors and proxy for all other requests. Now exposes [WebOfThings APIs](https://iot.mozilla.org/) for MySensors (supporting very few sensors at the moment) and on it's way to be a fully functional controller for MySensors. Contributions to support other sensors are welcome.

This server also acts as a proxy between Gateway and the Controller. Both might be either connected through a serial port or a TCP connection.

Before running the server, set the correct connection type and connection port for Gateway and Controller in conf.ini file.

WoT api's are exposed at `https://{host}:8888`

## To run the proxy server:
```
cargo run
```

## To install and run as a service in a debian/ubuntu flavour machine
- Add the following to your /etc/apt/sources.list system config file:
    ```bash
    echo "deb http://dl.bintray.com/tsathishkumar/myscontroller-rs vivid main" | sudo tee -a /etc/apt/sources.list
    ```
- Update the package list
    ```bash
    apt-get update
    ```
- Install the package
    ```bash
    apt install myscontroller-rs
    ```
- The configuration of the server can be found at the below location. 
    ```bash
    /etc/myscontroller-rs/conf.ini
    ```
    Example settings:
    ```bash
    encoding=utf-8

    [Gateway]
    type=TCP
    port=10.11.12.13:5003

    [Controller]
    type=TCP
    port=0.0.0.0:5003

    [Server]
    database_url=/var/lib/myscontroller-rs/sqlite.db
    ```
- Set up the right Gateway IP and Controller IP and restart the service.
    ```bash
    systemctl restart myscontroller-rs.service
    ```


Note: If you are using TCP for controller - the port value will be used to create TCP server listening on the specified port. (So it shoud be the address of the machine running MySController)

## TODO

- [x] Gracefully handle connection at both side, i.e never panic and wait for both connections
- [x] Ability to handle ota requests even when there is no controller connected
- [x] Ability to restart the node using api
- [x] Manage nodes and the firmwares installed, expose api's 
    - GET `/nodes`
    - POST `/nodes` payload 
    ```json
    {
        "node_id": 1,
        "node_name": "New switch",
        "firmware_type": 0,
        "firmware_version": 0,
        "desired_firmware_type": 10,
        "desired_firmware_version": 4,
        "auto_update": true,
        "scheduled": true
    }
    ```
    - PUT `/nodes` payload 
    ```json
    {
        "node_id": 1,
        "node_name": "New switch",
        "firmware_type": 0,
        "firmware_version": 0,
        "desired_firmware_type": 10,
        "desired_firmware_version": 4,
        "auto_update": true,
        "scheduled": true
    }
    ```
    - POST `/nodes/{node_id}/reboot`
- [x] Get node's firmware type and version from database and use it for ota request from node
- [x] Handle auto update flag in node 
    - whenever there is new version for a firmware, it should automatically update all nodes which have auto update as `true` to latest version
- [x] Manage firmwares type and version, ability to upload newer versions of firmwares, expose apis 
    - GET `/firmwares` - response `[{"firmware_type": 10, "firmware_version": 1, "firmware_name", "Blink"}]`
    - DELETE `/firmwares` - `[{"firmware_type": 10, "firmware_version": 1}]
    - POST `/firmwares` - payload `{ "firmware_type": 10, "firmware_version": 1, "firmware_name": "Blink", "firmware_file": <file>}`
    - PUT `/firmwares` - payload `{ "firmware_type": 10, "firmware_version": 1, "firmware_name": "Blink", "firmware_file": <file>}`
- [x] Improve error handling in api's (handling unique constraint in insert, updating unavailable firmwares etc)    
- [x] Node name support
- [x] Improve error handling across project (remove unwraps)
- [x] Improve logging (parsed message for OTA request etc)
- [x] Child sensors support
- [ ] Parse all the data and expose WoT API's using [webthing-rust](https://github.com/mozilla-iot/webthing-rust)
- [ ] Add swagger UI for node/firmware management APIs
- [ ] Add UI for node/firmware management    


## Future goals:

- [ ] MQTT support
- [ ] Store the "states" of each nodes - to make it standalone
- [ ] Beats/Telegraph support - to store "telemetri" data
