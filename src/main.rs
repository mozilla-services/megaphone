#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate failure;
extern crate config;
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

mod http;

use std::str;
use std::collections::{HashMap};
use std::env;

use config::{ConfigError, Config, Value, File};
use failure::Error;
use rusoto_core::{Region};
//use rusoto_sns::{Sns, SnsClient, GetTopicAttributesInput};
//use rusoto_sqs::{Sqs, SqsClient, ListQueuesRequest};

use http::create_rocket;

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

// ==== mysql

#[derive(Debug, Deserialize)]
struct DatabaseConfig {
    host: String,
    username: String,
    password: String,
    port: u16,
    tablename: String,
}

impl Default for DatabaseConfig {
    fn default() -> DatabaseConfig {
        DatabaseConfig {
            host: String::from("localhost"),
            username: String::from(""),
            password: String::from(""),
            port: 3306,
            tablename: String::from("megaphone_sevices"),
        }
    }
}

impl DatabaseConfig {
    fn dsn(&self) -> String {
        let mut user = String::from("");
        let mut port = String::from("");
        if (self.username.len() | self.password.len()) > 0 {
            user = format!("{}:{}@", self.username, self.password);
        }
        if self.port != 3306 {
            port = format!(":{}", self.port);
        }
        format!("mysql://{}{}{}", user, self.host, port)
    }
}

struct Database {
    config: DatabaseConfig
}

impl Database {
    fn new(config: DatabaseConfig) -> Database {
        Database {
            config: config,
        }
    }

    fn login(&self) -> mysql::Pool {
        mysql::Pool::new(self.config.dsn()).expect("Could not connect to database")
    }

    fn create(&self) -> Result<mysql::Pool, Error> {
        let pool = self.login();
        pool.prep_exec(r"create table if not exists :table_name (service_id text not null,
        version text not null, change_number integer not null, PRIMARY KEY('service_id')",
                       params!{"table_name" => self.config.tablename.clone()}
        )?;
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
            r"Insert into :table_name (service_id, version ) values (:service_id, :version)") {
            stmt.execute(
                params!{
                "table_name" => self.config.tablename.clone(),
                "service_id" => serviceid.clone(),
                "version" => version.clone()
                }
            ).expect("Could not add serviceid");
        }
        Ok(String::from("TBD: Increment"))
    }

    fn snapshot(&self) -> Result<HashMap<String, String>, Error> {
        let pool = self.login();
        let mut reply = HashMap::new();
        let content: Vec<(String, String)> = pool
            .prep_exec("select serviceid, version from :table_name",
                       params!("table_name" => self.config.tablename.clone()))
            .map(|result| result
                .map(|row | {
                        let rowdata = row.expect("Could not get row data");
                        let (serviceid, version) = mysql::from_row::<(String, String)>(rowdata);
                        (serviceid.clone(), version.clone())
                    })
                .collect())
            .unwrap();
        for (key, value) in content {
            reply.insert(key, value);
        }
        Ok(reply)
    }

    fn get(&self, serviceid: String) -> Result<String, Error> {
        let pool = self.login();
        let version: String = pool.prep_exec(
            "select version from :table_name where serviceid = :service_id limit 1",
            params! {
                "table_name"=>self.config.tablename.clone(),
                "service_id"=>serviceid
            })
            .map(|result| result
                .map(|row| {
                        let rowdata = row.expect("Could not get row data");
                        let version:String = mysql::from_row(rowdata);
                        version
                    })
                .collect()
            ).expect("Failed to fetch value");
        Ok(version)
    }
}

// ==== Config

#[derive(Debug, Deserialize)]
struct Settings {
    debug: bool,
    database: DatabaseConfig,
    aws: AwsConfig,
    // TODO: connection config
}

impl Settings {
    fn new() -> Result<Self, ConfigError> {
        let mut config = Config::new();

        // Read the default config file
        config.merge(File::with_name("config/default"))?;
        /*
        println!("RR: {:?}", match dict.get("port") {
            Some(v) => v.to_string(),
            None => String::from("undefined"),
        });
        */
        // Set the run mode to "dev" (or whatever is in RUN_MODE)
        let env = env::var("RUN_MODE").unwrap_or("dev".into());
        // pull in the run mode's configs (optional)
        config.merge(File::with_name(&format!("config/{}", env)).required(false))?;
        // And the local, optional config
        config.merge(File::with_name("config/local").required(false))?;
        // set defaults

        // Blech.

        let debug = match config.get_bool("debug") {
            Ok(v) => v,
            Err(_) => true,
        };
        let db_dict = match config.clone().get_table("database"){
            Ok(v) => v,
            Err(_) => HashMap::new(),
        };
        let database = DatabaseConfig {
            host: (match db_dict.get("host") {
                Some(v) => v.to_string(),
                None => String::from("localhost")
            }),
            port: (match db_dict.get("port") {
                Some(v) => v.to_string().parse::<u16>().expect("Invalid port number"),
                None => 3306
            }),
            username: (match db_dict.get("username") {
                Some(v) => v.to_string(),
                None => String::from("")
            }),
            password: (match db_dict.get("password") {
                Some(v) => v.to_string(),
                None => String::from("")
            }),
            tablename: (match db_dict.get("tablename") {
                Some(v) => v.to_string(),
                None => String::from("megaphone_services")
            }),
        };
        let aws_dict = match config.clone().get_table("aws") {
            Ok(v) => v,
            Err(_) => HashMap::new()
        };
        let aws = AwsConfig {
            region: (match aws_dict.get("region") {
                Some(v) => v.to_string(),
                None => String::from("uswest2")
            }).parse::<Region>().expect("Invalid region"),
            sns_topic: (match aws_dict.get("sns_topic") {
                Some(v) => v.to_string(),
                None => String::from("arn:aws:sns:us-west-2:927034868273:megaphone_updates")
            }),
            sqs_prefix: (match aws_dict.get("sqs_prefix") {
                Some(v) => v.to_string(),
                None => String::from("megaphone_")
            }),
        };
        Ok(Settings {
            debug: debug,
            database: database,
            aws: aws,
        })
    }
}

// ==== AWS
// # Replace with websocket/
#[derive(Debug, Deserialize)]
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
    config: AwsConfig,
}

impl AwsService {
    fn new(config: AwsConfig)-> AwsService {
        let services = AwsService{
            config: config,
        };
        // in spite of what https://github.com/rusoto/rusoto/blob/master/integration_tests/tests/sns.rs says,
        // these don't work. "simple()" is undefined.
        /*
        let sqs = SqsClient::simple(config.region);
        let sns = SnsClient::simple(config.region);

        let list_queue_request = ListQueuesRequest{
            queue_name_prefix:Some(config.sqs_prefix),
        };
        let queue_list = sqs.list_queues(&list_queue_request).expect("No sqs quees");
        println!("SQS Topics: {:?}", queue_list);
        if queue_list.len() == 0 {
            //TODO: Add a queue for this handler.
        }

        // sns_client.subscribe(SubscribeInput{endpoint: sqs_arn,
        //                                     protocol: "sqs",
        //                                     topic_arn: SNS_TOPIC_ARN});
        //
        let topic_arn = GetTopicAttributesInput{topic_arn: config.sns_topic};
        /*
            get connection
        */
        let topic = sns.get_topic_attributes(&topic_arn).expect("No such topic");
        // confirm a topic.
        println!("Topic {:?}", topic);
        */
        services
    }
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

    let config = Settings::new().expect("Could not get settings");
    println!("config {:?}", config);
    let aws_service = AwsService::new(config.aws);
    let database = Database::new(config.database);

    create_rocket().launch();
}
