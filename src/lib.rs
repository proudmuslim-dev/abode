#![feature(adt_const_params)]
#![allow(incomplete_features)]
#![warn(clippy::if_then_some_else_none)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate diesel;

pub mod db;
pub mod routes;
