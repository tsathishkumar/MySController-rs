#![feature(untagged_unions)]
#![feature(rustc_private)]

#[macro_use]
extern crate enum_primitive;
extern crate num;

extern crate serialport;
extern crate hex;
extern crate ihex;

extern crate crc;

extern crate crc16;

pub mod ota;
pub mod gateway;
pub mod proxy_controller;
pub mod message;
pub mod interceptor;

