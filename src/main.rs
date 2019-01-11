#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate mozsvc_common;
extern crate regex;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_derive;
extern crate slog_mozlog_json;
extern crate slog_term;
#[cfg(test)]
#[macro_use]
extern crate toml;

mod auth;
mod db;
mod error;
mod http;
mod logging;

fn main() {
    http::rocket().expect("rocket failed").launch();
}
