use std::collections::HashMap;
use std::io::Read;

use diesel::{replace_into, QueryDsl, RunQueryDsl};
use failure::ResultExt;
use rocket::{self, Data, Request, Rocket};
use rocket::data::{self, FromData};
use rocket::Outcome::*;
use rocket::outcome::IntoOutcome;
use rocket::request::{self, FromRequest};
use rocket_contrib::Json;

use db::{self, pool_from_config};
use db::models::Version;
use db::schema::versionv1;
use error::{HandlerError, HandlerErrorKind, HandlerResult, Result, VALIDATION_FAILED};

/// An authorized broadcaster
pub struct Broadcaster {
    pub id: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for Broadcaster {
    type Error = HandlerError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, HandlerError> {
        if let Some(_auth) = request.headers().get_one("Authorization") {
            // These should be guaranteed on the path when we're called
            let broadcaster_id = request
                .get_param::<String>(0)
                .map_err(|_| HandlerErrorKind::Unauthorized("Unexpected error".to_string()).into())
                .into_outcome(VALIDATION_FAILED)?;
            // TODO: Validate auth cookie
            Success(Broadcaster { id: broadcaster_id })
        } else {
            Failure((
                VALIDATION_FAILED,
                HandlerErrorKind::Unauthorized("Missing Authorization header".to_string()).into(),
            ))
        }
    }
}

/// An authorized reader of current broadcasts
//struct BroadcastAdmin;

// Version information from command line.
struct VersionInput {
    value: String,
}

impl FromData for VersionInput {
    type Error = HandlerError;

    fn from_data(_: &Request, data: Data) -> data::Outcome<Self, HandlerError> {
        let mut string = String::new();
        data.open()
            .read_to_string(&mut string)
            // XXX: lost the cause, might be nice for logging
            .map_err(|_| HandlerErrorKind::MissingVersionDataError.into())
            .into_outcome(VALIDATION_FAILED)?;
        if string.is_empty() {
            return Failure((
                VALIDATION_FAILED,
                HandlerErrorKind::InvalidVersionDataError.into(),
            ));
        }
        // TODO Validate the version info
        Success(VersionInput { value: string })
    }
}

/*
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
*/

// REST Functions

/// Set a version for a broadcaster / collection
#[post("/v1/broadcasts/<_broadcaster_id>/<collection_id>", data = "<version>")]
fn accept(
    _broadcaster_id: String,
    collection_id: String,
    version: HandlerResult<VersionInput>,
    broadcaster: HandlerResult<Broadcaster>,
    conn: db::Conn,
) -> HandlerResult<Json> {
    // TODO: improved error handling, logging+sentry

    let new_version = Version::new(broadcaster?, collection_id, version?.value);
    let _ = replace_into(versionv1::table)
        .values(&new_version)
        .execute(&*conn)
        .context(HandlerErrorKind::DBError)?;

    /*
    Ok(Json(MResponse {
        ..Default::default()
    }))
     */
    Ok(Json(json!({
        "status": 200,
    })))
}

/// Dump the nodes current version table
#[get("/v1/broadcasts")]
//fn dump(bcast_admin: BroadcastAdmin, conn: db::Conn) -> HandlerResult<Json> {
fn dump(conn: db::Conn) -> HandlerResult<Json> {
    // flatten into HashMap FromIterator<(K, V)>
    let collections: HashMap<String, String> = versionv1::table
        .select((versionv1::service_id, versionv1::version))
        .load(&*conn)
        .context(HandlerErrorKind::DBError)?
        .into_iter()
        .collect();
    Ok(Json(json!({ "collections": collections })))
}

#[error(404)]
fn not_found() -> HandlerResult<Json> {
    Err(HandlerErrorKind::NotFound.into())
}

pub fn rocket() -> Result<Rocket> {
    let rocket = rocket::ignite();
    let pool = pool_from_config(rocket.config())?;
    Ok(rocket
        .manage(pool)
        .mount("/", routes![accept, dump])
        .catch(errors![not_found]))
}

#[cfg(test)]
mod test {
    use std::env;

    use diesel::Connection;
    use rocket::local::Client;
    use rocket::http::{Header, Status};
    use rocket::response::Response;
    use serde_json::{self, Value};

    use db::Pool;
    use super::rocket;

    /// Return a Rocket Client for testing
    ///
    /// The managed db pool is set to a maxiumum of one connection w/
    /// a transaction began that is never committed
    fn rocket_client() -> Client {
        // hacky/easiest way to set into rocket's config
        env::set_var("ROCKET_DATABASE_POOL_MAX_SIZE", "1");
        let rocket = rocket().expect("rocket failed");
        {
            let pool = rocket.state::<Pool>().unwrap();
            let conn = &*pool.get().expect("Couldn't connect to database");
            conn.begin_test_transaction().unwrap();
        }
        Client::new(rocket).expect("rocket launch failed")
    }

    fn auth() -> Header<'static> {
        Header::new("Authorization".to_string(), "Bearer XXX".to_string())
    }

    fn json_body(response: &mut Response) -> Value {
        assert!(response.content_type().map_or(false, |ct| ct.is_json()));
        serde_json::from_str(&response.body_string().unwrap()).unwrap()
    }

    #[test]
    fn test_post() {
        let client = rocket_client();
        let mut response = client
            .post("/v1/broadcasts/foo/bar")
            .header(auth())
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let result = json_body(&mut response);
        assert_eq!(result.get("status").unwrap(), 200);
        assert_eq!(result.get("error"), None);
    }

    #[test]
    fn test_post_no_body() {
        let client = rocket_client();
        let mut response = client
            .post("/v1/broadcasts/foo/bar")
            .header(auth())
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let result = json_body(&mut response);
        // XXX:
        //assert_eq!(result.get("status").unwrap(), "ok");
        assert_eq!(result.get("status").unwrap(), Status::BadRequest.code);
        assert!(
            result
                .get("error")
                .unwrap()
                .as_str()
                .unwrap()
                .contains("Version")
        );
    }

    #[test]
    fn test_post_no_id() {
        let client = rocket_client();
        let mut response = client
            .post("/v1/broadcasts/foo")
            .header(auth())
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
        let result = json_body(&mut response);
        assert_eq!(result.get("status").unwrap(), 404);
        assert_eq!(result.get("error").unwrap().as_str().unwrap(), "Not Found");
    }

    #[test]
    fn test_post_no_auth() {
        let client = rocket_client();
        let mut response = client.post("/v1/broadcasts/foo/bar").body("v1").dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
        let result = json_body(&mut response);
        assert_eq!(result.get("status").unwrap(), 401);
    }

    #[test]
    fn test_post_get() {
        let client = rocket_client();
        let _ = client
            .post("/v1/broadcasts/foo/bar")
            .header(auth())
            .body("v1")
            .dispatch();
        let _ = client
            .post("/v1/broadcasts/baz/quux")
            .header(auth())
            .body("v0")
            .dispatch();
        let mut response = client.get("/v1/broadcasts").header(auth()).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let result = json_body(&mut response);
        let collections = result.get("collections").unwrap();
        assert_eq!(collections.as_object().map_or(0, |o| o.len()), 2);
        assert_eq!(collections.get("foo/bar").unwrap(), "v1");
        assert_eq!(collections.get("baz/quux").unwrap(), "v0");
    }

}
