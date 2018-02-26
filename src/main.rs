#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
//#[macro_use]
//extern crate serde_derive;
extern crate serde_json;

mod db;
mod error;
mod http;

use http::rocket;

fn main() {
    rocket().expect("rocket failed").launch();
}
