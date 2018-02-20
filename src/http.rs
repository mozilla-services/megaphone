use std::io::Read;

use rocket;
use rocket::{Request, Data, Rocket};
use rocket::data::{self, FromData};
use rocket::http::{Status};
use rocket::Outcome::*;
use rocket_contrib::Json;

use super::Settings;
use super::Database;
use db;

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
fn accept(broadcaster_id: String, collection_id: String, version: VersionInput, conn: db::Conn) -> Json<MResponse>{
    /// Set a version for a broadcaster / collection

    println!("broadcaster: {:?}\n\tcollection: {:?}\n\tversion:{:?}",
        broadcaster_id, collection_id, version.value
    );

    // TODO: Validate auth cookie
    // TODO: Validate broadcaster & collection; create SenderID
    // TODO: publish version change / update local table.

    use diesel::replace_into;
    use diesel::RunQueryDsl;
    use db::schema::versionv1::dsl::versionv1;
    use db::models::VersionV1;
    let new_version = VersionV1 {
        service_id: format!("{}/{}", broadcaster_id, collection_id),
        version: version.value
    };
    let results = replace_into(versionv1)
        .values(&new_version)
        .execute(&*conn)
        .expect("Error");

    Json(MResponse {..Default::default()})
}

/* Dump the current table */
#[get("/v1/rtu")]
fn dump(conn: db::Conn) -> Json<MResponse> {
    /// Dump the nodes current version info table.
    // TODO: dump the local table of senderID -> version data.
    let settings = Settings::new().expect("Could not get settings");
    let database = Database::new(settings.database);
    let snapshot = format!("{:?}", database.snapshot());

    //use diesel::query_dsl::SelectDsl;
    use diesel::QueryDsl;
    use diesel::RunQueryDsl;
    use db::models::*;
    use db::schema::versionv1::all_columns;
    use db::schema::versionv1::dsl::*;

    let results = versionv1.select(all_columns)
        .load::<VersionV1>(&*conn)
        .expect("Error loading Version records");
    eprintln!("results: {:?}", results);


    let response = MResponse {
        body: snapshot,
        ..Default::default()
    };
    Json(response)
}

// TODO: Database handler.
// TODO: PubSub handler.
// TODO: HTTP Error Handlers  https://rocket.rs/guide/requests/#error-catchers

pub fn create_rocket(pool_max_size: u32) -> Rocket {
    use db::init_pool;
    let pool = init_pool(pool_max_size);
    rocket::ignite()
        .manage(pool)
        .mount("/", routes![accept, dump])
}


#[cfg(test)]
mod test {
    use super::create_rocket;
    use rocket::local::Client;
    use rocket::http::Status;
    use serde_json;


    fn rocket_client() -> Client {
        use diesel::Connection;
        use db::Pool;
        let rocket = create_rocket(1);
        {
            let pool = rocket.state::<Pool>().expect("Expected a managed Pool");
            let conn = pool.get().expect("Expected a Connection from the Pool");
            (*conn).begin_test_transaction().expect("");
        }
        Client::new(rocket).expect("valid rocket instance")
    }

    #[test]
    fn hello_world() {
        //let rocket = create_rocket();
        //let client = Client::new(create_rocket()).expect("valid rocket instance");
        let client = rocket_client();

        let mut response = client
            .post("/v1/rtu/foo/bar")
            .body("v1")
            .dispatch();

        let mut response = client.get("/v1/rtu").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_string = response.body_string().expect("Expected a body");
        eprintln!("got: {}",  body_string);
        //let body = serde_json::from_str(body_string.as_str());
        //assert_eq!(, Some("Hello, world!".into()));
    }

    #[test]
    fn hello_world2() {
        //let client = Client::new(create_rocket()).expect("valid rocket instance");
        let client = rocket_client();
        let mut response = client
            .post("/v1/rtu/foo/bar")
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body_string = response.body_string().expect("Expected a body");
        //let body = serde_json::from_str(body_string.as_str());
        //assert_eq!(, Some("Hello, world!".into()));
    }
}
