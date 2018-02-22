use std::io::Read;

use rocket;
use rocket::{Request, Data, Rocket};
use rocket::data::{self, FromData};
use rocket::http::{Status};
use rocket::Outcome::*;
use rocket_contrib::Json;

use super::Settings;
use super::Database;

// Version information from command line.
struct VersionInput {
    value: String,
}

// ==== REST

impl FromData for VersionInput {
    type Error = String;

    fn from_data(_req: &Request, data: Data) -> data::Outcome<Self, String> {
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
#[derive(Serialize)]
struct MResponse {
    status: u8,
    error_code: u8,
    error: String,
    body: String,
}

impl Default for MResponse {
    fn default() -> MResponse {
        MResponse {
            status: 200,
            error_code: 0,
            error: String::from(""),
            body: String::from(""),
        }
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
    let settings = Settings::new().expect("Could not get settings");
    let database = Database::new(settings.database);
    let snapshot = format!("{:?}", database.snapshot());
    let response = MResponse {
        body: snapshot,
        ..Default::default()
    };
    Json(response)
}

// TODO: Database handler.
// TODO: PubSub handler.
// TODO: HTTP Error Handlers  https://rocket.rs/guide/requests/#error-catchers

pub fn create_rocket() -> Rocket {
    rocket::ignite().mount("/", routes![accept, dump])
}

#[cfg(test)]
mod test {
    use super::create_rocket;
    use rocket::local::Client;
    use rocket::http::Status;
    use serde_json;

    #[test]
    fn hello_world() {
        let client = Client::new(create_rocket()).expect("valid rocket instance");
        let mut response = client.get("/v1/rtu").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_string = response.body_string().expect("Expected a body");
        //let body = serde_json::from_str(body_string.as_str());
        //assert_eq!(, Some("Hello, world!".into()));
    }
}
