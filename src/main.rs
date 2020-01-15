#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate rocket;

mod auth;
mod db;
mod error;
mod http;
mod logging;

fn main() {
    http::rocket().expect("rocket failed").launch();
}
