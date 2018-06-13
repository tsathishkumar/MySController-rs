extern crate actix;
extern crate actix_web;
extern crate crc16;
extern crate http;
#[macro_use]
extern crate enum_primitive;
extern crate chrono;
extern crate hex;
extern crate ihex;
extern crate num;
extern crate serialport;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate crossbeam_channel as channel;
extern crate r2d2;
extern crate serde_json;
extern crate futures;
extern crate bytes;

pub mod api;
pub mod core;
pub mod handler;
pub mod model;
