#![feature(untagged_unions)]
#![feature(rustc_private)]
#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate crc16;
#[macro_use]
extern crate enum_primitive;
extern crate hex;
extern crate ihex;
extern crate num;
extern crate serialport;
extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate rocket;
extern crate rocket_contrib;
extern crate r2d2;

pub mod firmware;
pub mod gateway;
pub mod interceptor;
pub mod message;
pub mod ota;
pub mod proxy;
pub mod schema;
pub mod node;
pub mod api;
pub mod pool;