#![feature(plugin)]
#![plugin(rocket_codegen)]

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
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
// prefer slog_<level> names to avoid conflicting w/ rocket's error!. rocket
// 0.4 will rename it to catcher
#[macro_use(
    slog_b, slog_debug, slog_log, slog_kv, slog_info, slog_o, slog_record, slog_record_static,
    slog_warn
)]
extern crate slog;
extern crate slog_async;
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
