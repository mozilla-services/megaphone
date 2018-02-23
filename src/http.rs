use std::collections::HashMap;
use std::io::Read;

use diesel::{replace_into, QueryDsl, RunQueryDsl};
use rocket::{self, Data, Request, Rocket};
use rocket::data::{self, FromData};
use rocket::http::Status;
use rocket::Outcome::*;
use rocket_contrib::Json;

use db::{self, init_pool};
use db::models::Version;
use db::schema::versionv1::all_columns;
use db::schema::versionv1::dsl::versionv1;

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

        Success(VersionInput { value: string })
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
fn accept(
    broadcaster_id: String,
    collection_id: String,
    version: VersionInput,
    conn: db::Conn,
) -> Json<MResponse> {
    /// Set a version for a broadcaster / collection
    // TODO: Validate auth cookie
    // TODO: Validate broadcaster & collection; create SenderID
    // ^H^H^H^HTODO: publish version change / update local table.

    // TODO: improved error handling

    let new_version = Version {
        service_id: format!("{}/{}", broadcaster_id, collection_id),
        version: version.value,
    };
    let results = replace_into(versionv1)
        .values(&new_version)
        .execute(&*conn)
        .expect("Error");

    Json(MResponse {
        ..Default::default()
    })
}

/* Dump the current table */
#[get("/v1/rtu")]
fn dump(conn: db::Conn) -> Json {
    // Dump the nodes current version info table.
    let collections: HashMap<String, String> = versionv1
        .select(all_columns)
        .load(&*conn)
        .expect("Error loading Version records")
        .into_iter()
        .collect();
    Json(json!({ "collections": collections }))
}

// TODO: Database handler.
// TODO: PubSub handler.
// TODO: HTTP Error Handlers  https://rocket.rs/guide/requests/#error-catchers

pub fn create_rocket(pool_max_size: u32) -> Rocket {
    let rocket = rocket::ignite();
    let database_url = rocket
        .config()
        .get_str("database_url")
        .expect("ROCKET_DATABASE_URL undefined")
        .to_string();
    //let pool_max_size = rocket.config().get_int("max_pool_size").unwrap_or(10) as u32;
    let pool = init_pool(database_url, pool_max_size);
    rocket.manage(pool).mount("/", routes![accept, dump])
}

#[cfg(test)]
mod test {
    use diesel::Connection;
    use rocket::local::Client;
    use rocket::http::Status;
    use serde_json::{self, Value};

    use db::Pool;
    use super::create_rocket;

    /// Return a Rocket Client for testing
    ///
    /// The managed db pool is set to a maxiumum of one connection w/
    /// a transaction began that is never committed
    fn rocket_client() -> Client {
        let rocket = create_rocket(1);
        {
            let pool = rocket.state::<Pool>().expect("Expected a managed Pool");
            let conn = &*pool.get().expect("Expected a Connection from the Pool");
            conn.begin_test_transaction().expect("");
        }
        Client::new(rocket).expect("valid rocket instance")
    }

    #[test]
    fn test_post_get() {
        let client = rocket_client();
        let _ = client.post("/v1/rtu/foo/bar").body("v1").dispatch();
        let _ = client.post("/v1/rtu/baz/quux").body("v0").dispatch();
        let mut response = client.get("/v1/rtu").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.body_string().unwrap();
        assert!(response.content_type().map_or(false, |ct| ct.is_json()));

        let result: Value = serde_json::from_str(&body).unwrap();
        let collections = result.as_object().unwrap().get("collections").unwrap();
        assert_eq!(collections.as_object().map_or(0, |o| o.len()), 2);
        assert_eq!(collections.get("foo/bar").unwrap(), "v1");
        assert_eq!(collections.get("baz/quux").unwrap(), "v0");
    }

    #[test]
    fn test_post() {
        let client = rocket_client();
        let mut response = client.post("/v1/rtu/foo/bar").body("v1").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let body = response.body_string().unwrap();
        assert!(response.content_type().map_or(false, |ct| ct.is_json()));

        let result: Value = serde_json::from_str(&body).unwrap();
        assert!(result.is_object());
        // XXX:
        //assert_eq!(result.get("status").unwrap(), "ok");
        assert_eq!(result.get("status").unwrap(), 200);
        // XXX:
        //assert_eq!(result.get("error"), None);
    }
}
