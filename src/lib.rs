#![feature(untagged_unions)]
#![feature(rustc_private)]

#[macro_use]
extern crate enum_primitive;
extern crate num;

extern crate hex;
extern crate ihex;
extern crate serialport;

extern crate crc;

extern crate crc16;

pub mod gateway;
pub mod interceptor;
pub mod message;
pub mod ota;
pub mod proxy_controller;
