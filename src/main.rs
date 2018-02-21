#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate failure;
extern crate hyper;
extern crate hyper_tls;
extern crate rocket;
extern crate rocket_contrib;
extern crate diesel;
#[macro_use] extern crate mysql;
extern crate rusoto_core;
extern crate rusoto_sns;
extern crate rusoto_sqs;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde;

use std::io::Read;
use std::str;
use std::collections;

use hyper::client::Connect;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use failure::Error;
use rocket::{Request, Data, Outcome, Rocket};
use rocket::data::{self, FromData};
use rocket::http::{Status};
use rocket::Outcome::*;
use rocket_contrib::Json;
use rusoto_core::{DefaultCredentialsProvider, Region, default_tls_client};
use rusoto_core::request::DispatchSignedRequest;
use rusoto_sns::*;
use rusoto_sqs::*;
use mysql;
use serde::ser::{Serialize, Serializer, SerializeStruct};

//TODO: divide into sane components.

// ==== Error Handling ( see https://boats.gitlab.io/blog/post/2017-11-30-failure-0-1-1/)

#[derive(Debug, Fail)]
enum MegaphoneError {
    #[fail(display = "{}: Invalid Version info (must be URL safe Base 64)", name)]
    InvalidVersionDataError {
        name: String,
    },

    #[fail(display = "{}: Version information not included in body of update", name)]
    MissingVersionDataError {
        name: String,
    },

}

// ==== PubSub
// TODO: create new topic if not present?


// Version information from command line.
struct VersionInput {
    value: String,
}
// ==== mysql

struct MysqlConfig {
    host: String,
    username: String,
    password: String,
    port: u16,
    tablename: String,
}

impl Default for MysqlConfig {
    fn default() -> MysqlConfig {
        MysqlConfig {
            host: "localhost",
            username: "",
            password: "",
            port: 3306,
            tablename: "megaphone_sevices",
        }
    }
}

impl MysqlConfig {
    fn dsn(&self) -> String {
        let mut user = String.from("");
        let mut port = String.from("");
        if self.username || self.password {
            user = format!("{}:{}@", self.username, self.password);
        }
        if self.port != 3306 {
            port = format!(":{}", self.port);
        }
        format!("mysql://{}{}{}", user, self.host, port)
    }
}

struct Database {
    config: MysqlConfig
}

impl Database {
    fn login(mut self) -> mysql::Pool {
        mysql::Pool::new(self.config.dsn()).expect("Could not connect to database")
    }

    fn create(&self) -> Result<mysql::Pool, Error> {
        pool = self.login();
        pool.prep_exec(r"create table if not exists :table_name (service_id text not null,
        version text not null, change_number integer not null, PRIMARY KEY('service_id')",
                       params!{"table_name" => self.config.tablename}
        );
        Ok(pool)
    }

    fn store(&self, serviceid: String, version: String) -> Result<String, Error> {
        // TODO: Get a transaction number out of mysql
        let pool = self.login();
        /*
        // TODO: create this in rust...
        start transaction;
        select @change := max(change_number) + 1 from :table_name;
        insert into :table_name (service_id, version, change_number) values (:service, :version,
        @change);
        commit;
        select @change; // return the change number
        */
        for mut stmt in pool.prepare(
            r"Insert into :table_name (service_id, version ) values (:service_id,:version)") {
            let version = stmt.execute(
                params!{
                "table_name" => self.config.tablename,
                "service_id" => serviceid,
                "version" = version
                }
            ).expect("Could not add serviceid");
            Ok(version)
        }
    }

    fn snapshot(&self) -> Result<HashMap<String, String>, Error> {
        let result :Hashmap<String, String> = HashMap::new();
        let pool = self.login();
        let items = pool
            .prep_exec("select serviceid, version from :table_name",
                       params!("table_name", self.config.tablename))
            .map(|result| result
                .map(|row | {
                    let (serviceid, version) = mysql::from_row(row);
                    result.insert(serviceid, version);
                    }));
        Ok(results)
    }

    fn get(&self, serviceid: String) -> Result<String, Error> {
        pool = self.login();
        let version: String = pool.prep_exec(
            "select version from :table_name where serviceid = :service_id limit 1",
            params! {
                "table_name"=>self.config.tablename,
                "service_id"=>serviceid
            })
            .map(|result| {
                result
                    .expect("Failed to query database")
                    .map(|row: mysql::Row| {
                        let (version) = mysql::from_row(row);
                        version
                    })
                    .collect()
            }).expect("Failed to fetch value");
        Ok(version)
    }
}

// ==== AWS
struct AwsConfig {
    region: Region,
    sns_topic: String,
    sqs_prefix: String,

}

impl Default for AwsConfig {
    fn default() -> AwsConfig {
        AwsConfig {
            region: Region::UsWest2,
            sns_topic: String::from("arn:aws:sns:us-west-2:927034868273:megaphone_updates"),
            sqs_prefix: String::from("megaphone_"),
        }
    }
}

struct AwsService {
    sns: SnsClient<DefaultCredentialsProvider, RequestDispatcher>,
    sqs: SqsClient<DefaultCredentialsProvider, RequestDispatcher>,
    config: AwsConfig,
}

impl AwsService {
    fn new(config: &AwsConfig)-> AwsService {
        let dispatcher = default_tls_client()?;
        let credentials = DefaultCredentialsProvider::new()?;
        let reply = AwsService{
            config: config.clone(),
            sqs: SqsClient::new(
                dispatcher.clone(),
                credentials.clone(),
                region.clone()),
            sns: SnsClient::new(
                dispatcher.clone(),
                credentials.clone(),
                region.clone()),
        };

        {
            // read the latest versions from the db;
            let scan = ScanInput{
                attributes_to_get: vec![String::from("serviceid"), String::from("version")],
                table_name: String::from("megaphone_states"),
                ..Default::default()
            };
            let output = services.ddb.scan(&scan).expect("Could not scan service table");
            for item in output.items {
                let record = VersionRecord{
                    change: 0,
                    sender_id: item.sender_id,
                    version: item.version,
                };
                versions.append(record);
            }

        }
        // init the sqs client
        let list_queue_request = ListQueuesRequest{
            queue_name_prefix:Some(config.sqs_prefix),
        };
        let queue_list = services.sqs.list_queues(&list_queue_request).expect("No sqs quees");
        println!("SQS Topics: {:?}", queue_list);
        if queue_list.len() == 0 {
            //TODO: Add a queue for this handler.
        }

        // sns_client.subscribe(SubscribeInput{endpoint: sqs_arn,
        //                                     protocol: "sqs",
        //                                     topic_arn: SNS_TOPIC_ARN});
        //
        let topic_arn = GetTopicAttributesInput{topic_arn:SNS_TOPIC_ARN};
        /*
            get connection
        */
        let topic = services.sns.get_topic_attributes(&topic_arn).expect("No such topic");
        // confirm a topic.
        println!("Topic {:?}", topic);

        reply
    }
}

// ==== REST

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

fn main() {
    // local cache
    let versions = VersionTable{
        items: VecQueue::new(),
        change_record: 0,
    };
    let aws_service = AwsService::new(AwsConfig(..Default.default()));

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
