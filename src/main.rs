#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate failure;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[cfg(test)]
#[macro_use]
extern crate toml;

mod auth;
mod db;
mod error;
mod http;

fn main() {
    http::rocket().expect("rocket failed").launch();
}
