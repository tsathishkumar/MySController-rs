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
extern crate diesel_derive_enum;
#[macro_use]
extern crate serde_derive;
extern crate bytes;
extern crate crossbeam_channel as channel;
extern crate futures;
extern crate r2d2;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate log;
extern crate webthing;

pub mod api;
pub mod core;
pub mod handler;
pub mod model;
pub mod wot;
