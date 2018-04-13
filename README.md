# MyRController
Proxy controller for MySensors. It is to perform OTA firmware updates, and proxy all other requests to the actual controllers like homeassist. Mainly to add OTA support for homeassist controller, but can work with any other controllers.

This server acts as a proxy between Gateway and the Controller. Both might be either connected through a serial port or a TCP connection.

Before running the server, set the connection type and connection port for Gateway and Controller.

```
export GATEWAY_CONNECTION=SERIAL
export CONTROLLER_CONNECTION=TCP`
export GATEWAY_PORT=/dev/tty1
export CONTROLLER_PORT=0.0.0.0:5003
cargo run
```
