#![feature(untagged_unions)]
#![feature(rustc_private)]
#![feature(plugin)]
#![feature(decl_macro)]
#![plugin(rocket_codegen)]
#![feature(slice_patterns)]

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
extern crate crossbeam_channel as channel;
extern crate multipart;

pub mod api;
pub mod core;
pub mod model;
pub mod handler;