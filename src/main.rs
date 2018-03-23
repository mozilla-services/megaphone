#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;

mod auth;
mod db;
mod error;
mod http;

use http::rocket;

fn main() {
    rocket().expect("rocket failed").launch();
}
