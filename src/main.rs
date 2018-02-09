#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate diesel;
extern crate serde_json;
extern crate websocket;

/* Set a version */
#[post("/v1/rtu/<broadcaster_id>/<collection_id>")]
fn accept(broadcaster_id: String, collection_id: String) -> String {
    return String::from("Hello, Other world");
}

/* Dump the current table */
#[get("/v1/rtu")]
fn dump() -> String {
    return String::from("Hello, Other world");
}

// TODO: Websocket handler.

fn main() {
    println!("Hello world.");
}
