#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate error_chain;
extern crate hyper;
extern crate hyper_tls;
extern crate rocket;
extern crate rocket_contrib;
extern crate diesel;
extern crate rusoto_core;
extern crate rusoto_sns;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

mod errors;

use std::io::Read;

use hyper::client::Connect;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use errors::*;
use rocket::{Request, Data, Outcome, Rocket};
use rocket::data::{self, FromData};
use rocket::http::{Status};
use rocket::Outcome::*;
use rocket_contrib::Json;
use rusoto_core::{DefaultCredentialsProvider, Region, default_tls_client};
use rusoto_core::request::DispatchSignedRequest;
use rusoto_sns::*;
use serde::ser::{Serialize, Serializer, SerializeStruct};

//TODO: divide into sane components.

// ==== PubSub
// TODO: create new topic if not present?




// ==== Version
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
#[derive(Serialize)]
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

/*
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
*/

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

fn create_rocket() -> Rocket {
    rocket::ignite().mount("/", routes![accept, dump])
}

/*
    We need to use SNS for fan out broadcast of updates to the other nodes. Each node will also
    have an SQS queue that it will have to subscribe to the SNS fan-out. The node reads pending
    messages off of SQS.

    Note, you'll need to poll for messages from SQS.
*/

//fn start_aws<C>() -> Result<SnsClient<DefaultCredentialsProvider, hyper::Client<C>>>
//    where C: Connect
//fn start_aws<C>() -> Result<SnsClient<DefaultCredentialsProvider, hyper::Client<HttpsConnector<HttpConnector>>>>
fn start_aws() -> Result<Box<Sns>>
{
    const SNS_TOPIC_ARN: &str = "arn:aws:sns:us-west-2:927034868273:megaphone_updates";  //TODO: config

    let provider = DefaultCredentialsProvider::new()?;
    let client = SnsClient::new(default_tls_client().unwrap(), provider, Region::UsWest2);
    //let topic_arn = GetTopicAttributesInput{topic_arn=SNS_TOPIC_ARN};
    /*
        get connection
        topic = client.get_topic_attributes(arn).expect("No such topic")
        // confirm a topic.


    */

    Ok(Box::new(client))

}
fn main() {
    create_rocket().launch();
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
