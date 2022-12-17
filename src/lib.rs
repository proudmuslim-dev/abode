#![feature(adt_const_params)]
#![allow(incomplete_features)]
#![warn(
    clippy::if_then_some_else_none,
    clippy::string_to_string,
    clippy::str_to_string,
    clippy::branches_sharing_code,
    clippy::unused_self
)]
#![deny(rust_2018_idioms)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

pub mod db;
pub mod routes;
