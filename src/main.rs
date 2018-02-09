#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate diesel;
extern crate serde_json;
extern crate serde;

use std::io::Read;

use rocket::{Request, Data, Outcome};
use rocket::data::{self, FromData};
use rocket::http::{Status};
use rocket::Outcome::*;
use rocket_contrib::Json;
use serde::ser::{Serialize, Serializer, SerializeStruct};

//TODO: divide into sane components.

struct VersionTable {
    // TODO: find the best version storage table.

}

struct VersionRecord {
    update: u64,        // the update instance
    sender_id: String,  // combination of the broadcast + '/' + collection ids
    version: String,    // version information
}

// Version information from command line.
struct VersionInput {
    value: String,
}

impl FromData for VersionInput {
    type Error = String;

    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        let mut string = String::new();
        if let Err(e) = data.open().read_to_string(&mut string) {
            return Failure((Status::InternalServerError, format!("{:?}", e)));
        }

        // TODO Validate the version info

        Success(VersionInput {
            value: string
        })
    }
}


// Generic Response
struct MResponse {
    status: u8,
    error_code: u8,
    error: String,
}

impl Default for MResponse {
    fn default() -> MResponse {
        MResponse {
            status: 200,
            error_code: 0,
            error: String::from("")
        }
    }
}

impl Serialize for MResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut s = serializer.serialize_struct("MResponse", 3)?;
        s.serialize_field("status", &self.status)?;
        if &self.error_code > &u8::from(0) {
            s.serialize_field("error_code", &self.error_code)?;
            s.serialize_field("error", &self.error)?;
        }
        s.end()
    }
}

// REST Functions
#[post("/v1/rtu/<broadcaster_id>/<collection_id>", data = "<version>")]
fn accept(broadcaster_id: String, collection_id: String, version: VersionInput) -> Json<MResponse>{
    /// Set a version for a broadcaster / collection
    println!("broadcaster: {:?}\n\tcollection: {:?}\n\tversion:{:?}",
        broadcaster_id, collection_id, version.value
    );
    // TODO: Validate auth cookie
    // TODO: Validate broadcaster & collection; create SenderID
    // TODO: publish version change / update local table.
    Json(MResponse {..Default::default()})
}

/* Dump the current table */
#[get("/v1/rtu")]
fn dump() -> Json<MResponse> {
    /// Dump the nodes current version info table.
    // TODO: dump the local table of senderID -> version data.
    Json(MResponse {..Default::default()})
}

// TODO: Database handler.
// TODO: PubSub handler.
// TODO: HTTP Error Handlers  https://rocket.rs/guide/requests/#error-catchers

fn main() {
    rocket::ignite().mount("/", routes![accept, dump]).launch();
}
