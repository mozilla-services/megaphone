use std::env;
use std::io::Read;

use diesel::{dsl::sql, sql_types::Integer, OptionalExtension, QueryDsl, RunQueryDsl};
use failure::ResultExt;
use lazy_static::lazy_static;
use regex::Regex;
use rocket::{
    self,
    config::RocketConfig,
    data::{self, FromDataSimple},
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest},
    response::{content, status},
    Data,
    Outcome::{Failure, Success},
    Request, Rocket,
};
use rocket_contrib::{json, json::JsonValue};
use slog::{error, info};

use crate::auth;
use crate::db::{
    self,
    models::{Broadcaster, Reader},
    schema::broadcastsv1,
};
use crate::error::{HandlerError, HandlerErrorKind, HandlerResult, Result, VALIDATION_FAILED};
use crate::logging::{self, RequestLogger};

lazy_static! {
    static ref URLSAFE_B64_RE: Regex = Regex::new(r"^[A-Za-z0-9\-_]+$").unwrap();
}

impl<'a, 'r> FromRequest<'a, 'r> for Broadcaster {
    type Error = HandlerError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, HandlerError> {
        auth::authorized_broadcaster(request).into_outcome(VALIDATION_FAILED)
    }
}

/// A new broadcast
#[derive(Debug)]
struct VersionInput {
    value: String,
}

impl FromDataSimple for VersionInput {
    type Error = HandlerError;

    fn from_data(_: &Request<'_>, data: Data) -> data::Outcome<Self, HandlerError> {
        let mut value = String::new();
        data.open()
            .read_to_string(&mut value)
            .context(HandlerErrorKind::MissingVersionDataError)
            .map_err(Into::into)
            .into_outcome(VALIDATION_FAILED)?;
        if value.is_empty() || value.len() > 200 || !value.is_ascii() {
            return Failure((
                VALIDATION_FAILED,
                HandlerErrorKind::InvalidVersionDataError.into(),
            ));
        }
        Success(VersionInput { value })
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Reader {
    type Error = HandlerError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, HandlerError> {
        auth::authorized_reader(request).into_outcome(VALIDATION_FAILED)
    }
}

// REST Functions

/// Set a version for a broadcaster / bchannel
#[put("/v1/broadcasts/<broadcaster_id>/<bchannel_id>", data = "<version>")]
fn broadcast(
    conn: HandlerResult<db::Conn>,
    log: RequestLogger,
    broadcaster: HandlerResult<Broadcaster>,
    broadcaster_id: String,
    bchannel_id: String,
    version: HandlerResult<VersionInput>,
) -> HandlerResult<status::Custom<JsonValue>> {
    let conn = conn?;

    if broadcaster_id.len() > 64 || !URLSAFE_B64_RE.is_match(&broadcaster_id) {
        Err(HandlerErrorKind::InvalidBroadcasterId)?
    }
    if bchannel_id.len() > 128 || !URLSAFE_B64_RE.is_match(&bchannel_id) {
        Err(HandlerErrorKind::InvalidBchannelId)?
    }

    let version = version?.value;
    let created = broadcaster?.broadcast_new_version(&conn, &bchannel_id, &version)?;
    let status = if created { Status::Created } else { Status::Ok };
    info!(
        log,
        "Broadcast: {}/{} new version: {}",
        broadcaster_id,
        bchannel_id,
        &version;
        "code" => status.code
    );
    Ok(status::Custom(
        status,
        json!({
            "code": status.code
        }),
    ))
}

/// Dump the current version table
#[get("/v1/broadcasts")]
fn get_broadcasts(
    conn: HandlerResult<db::Conn>,
    reader: HandlerResult<Reader>,
) -> HandlerResult<JsonValue> {
    let conn = conn?;
    let broadcasts = reader?.read_broadcasts(&conn)?;
    Ok(json!({
        "code": 200,
        "broadcasts": broadcasts
    }))
}

#[get("/__version__")]
fn version() -> content::Json<&'static str> {
    content::Json(include_str!("../version.json"))
}

#[get("/__heartbeat__")]
fn heartbeat(conn: HandlerResult<db::Conn>, log: RequestLogger) -> status::Custom<JsonValue> {
    let result = conn.and_then(|conn| {
        Ok(broadcastsv1::table
            .select(sql::<Integer>("1"))
            .get_result::<i32>(&*conn)
            .optional()
            .context(HandlerErrorKind::DBError)?)
    });

    let status = match result {
        Ok(_) => Status::Ok,
        Err(e) => {
            let status = Status::ServiceUnavailable;
            error!(log, "Database heartbeat failed: {}", e; "code" => status.code);
            status
        }
    };

    let msg = if status == Status::Ok { "ok" } else { "error" };
    status::Custom(
        status,
        json!({
            "status": msg,
            "code": status.code,
            "database": msg,
        }),
    )
}

#[get("/__lbheartbeat__")]
fn lbheartbeat() {}

#[catch(404)]
fn not_found() -> HandlerResult<()> {
    Err(HandlerErrorKind::NotFound)?
}

pub fn rocket() -> Result<Rocket> {
    // RocketConfig::init basically
    let rconfig = RocketConfig::read().unwrap_or_else(|_| {
        let path = env::current_dir()
            .unwrap()
            .join(&format!(".{}.{}", "default", "Rocket.toml"));
        RocketConfig::active_default_from(Some(&path)).unwrap()
    });

    // rocket::ignite basically
    let config = rconfig.active().clone();
    setup_rocket(rocket::custom(config))
}

fn setup_rocket(rocket: Rocket) -> Result<Rocket> {
    let pool = db::pool_from_config(rocket.config())?;
    let authenticator = auth::BearerTokenAuthenticator::from_config(rocket.config())?;
    let environment = rocket.config().environment;
    let logger = logging::init_logging(rocket.config())?;
    db::run_embedded_migrations(rocket.config())?;
    Ok(rocket
        .manage(pool)
        .manage(authenticator)
        .manage(environment)
        .manage(logger)
        .mount(
            "/",
            routes![broadcast, get_broadcasts, version, heartbeat, lbheartbeat],
        )
        .register(catchers![not_found]))
}

#[cfg(test)]
mod test {
    use rocket;
    use rocket::config::{Config, Environment, RocketConfig};
    use rocket::http::{Header, Status};
    use rocket::local::Client;
    use rocket::response::Response;
    use rocket_contrib::json;
    use serde_json::{self, Value};
    use toml::{toml, toml_internal};

    use super::setup_rocket;

    /// Test auth headers
    enum Auth {
        Foo,
        FooAlt,
        Baz,
        Reader,
    }

    impl Into<Header<'static>> for Auth {
        fn into(self) -> Header<'static> {
            let token = match self {
                Auth::Foo => "feedfacedeadbeef",
                Auth::FooAlt => "deadbeeffacefeed",
                Auth::Baz => "baada555deadbeef",
                Auth::Reader => "00000000deadbeef",
            };
            Header::new("Authorization".to_string(), format!("Bearer {}", token))
        }
    }

    /// Return a Rocket Client for testing
    ///
    /// The managed db pool is set to a maxiumum of one connection w/
    /// a transaction began that is never committed
    fn rocket_client() -> Client {
        // create a separate test config but inheriting database_url
        let rconfig = RocketConfig::read().expect("reading rocket Config failed");
        let database_url = rconfig
            .active()
            .get_str("database_url")
            .expect("ROCKET_DATABASE_URL undefined");

        let config = Config::build(Environment::Development)
            .extra("database_url", database_url)
            .extra("database_pool_max_size", 1)
            .extra("database_use_test_transactions", true)
            .extra("json_logging", false)
            .extra(
                "broadcaster_auth",
                toml! {
                    foo = ["feedfacedeadbeef", "deadbeeffacefeed"]
                    baz = ["baada555deadbeef"]
                },
            )
            .extra("reader_auth", toml! {reader = ["00000000deadbeef"]})
            .unwrap();

        let rocket = setup_rocket(rocket::custom(config)).expect("rocket failed");
        Client::new(rocket).expect("rocket launch failed")
    }

    fn json_body(response: &mut Response<'_>) -> Value {
        assert!(response.content_type().map_or(false, |ct| ct.is_json()));
        serde_json::from_str(&response.body_string().unwrap()).unwrap()
    }

    #[test]
    fn test_put() {
        let client = rocket_client();
        let mut response = client
            .put("/v1/broadcasts/foo/bar")
            .header(Auth::Foo)
            .body("v0")
            .dispatch();
        assert_eq!(response.status(), Status::Created);
        assert_eq!(json_body(&mut response), *json!({"code": 201}));
        let mut response = client
            .put("/v1/broadcasts/foo/bar")
            .header(Auth::FooAlt)
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(json_body(&mut response), *json!({"code": 200}));
    }

    #[test]
    fn test_put_no_body() {
        let client = rocket_client();
        let mut response = client
            .put("/v1/broadcasts/foo/bar")
            .header(Auth::Foo)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let result = json_body(&mut response);
        assert_eq!(result["code"], Status::BadRequest.code);
        assert!(result["error"].as_str().unwrap().contains("Version"));
    }

    #[test]
    fn test_put_no_id() {
        let client = rocket_client();
        let mut response = client
            .put("/v1/broadcasts/foo")
            .header(Auth::Foo)
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::NotFound);
        assert_eq!(
            json_body(&mut response),
            *json!({"code": 404, "errno": 123, "error": "Not Found"})
        );
    }

    #[test]
    fn test_put_no_auth() {
        let client = rocket_client();
        let mut response = client.put("/v1/broadcasts/foo/bar").body("v1").dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
        assert!(response
            .headers()
            .get_one("WWW-Authenticate")
            .unwrap()
            .starts_with("Bearer "));
        let result = json_body(&mut response);
        assert_eq!(result["code"], 401);
    }

    #[test]
    fn test_put_bad_auth() {
        let client = rocket_client();
        let mut response = client
            .put("/v1/broadcasts/foo/bar")
            .header(Auth::Baz)
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::Forbidden);
        let result = json_body(&mut response);
        assert_eq!(result["code"], 403);
    }

    #[test]
    fn test_put_bad_ids() {
        let client = rocket_client();
        let mut response = client
            .put("/v1/broadcasts/foo+bar/baz")
            .header(Auth::Baz)
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let result = json_body(&mut response);
        assert_eq!(result["code"], 400);
        assert!(result["error"].as_str().unwrap().contains("broadcasterID"));

        let mut response = client
            .put("/v1/broadcasts/foo/bar+baz")
            .header(Auth::Baz)
            .body("v1")
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let result = json_body(&mut response);
        assert_eq!(result["code"], 400);
        assert!(result["error"].as_str().unwrap().contains("bchannelID"));
    }

    #[test]
    fn test_put_bad_version() {
        let client = rocket_client();
        let mut response = client
            .put("/v1/broadcasts/foo/bar")
            .header(Auth::Baz)
            .body("v1".repeat(101))
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        let result = json_body(&mut response);
        assert_eq!(result["code"], 400);
        assert!(result["error"].as_str().unwrap().contains("Version"));
    }

    #[test]
    fn test_get_no_auth() {
        let client = rocket_client();
        let mut response = client.get("/v1/broadcasts").dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
        assert!(response
            .headers()
            .get_one("WWW-Authenticate")
            .unwrap()
            .starts_with("Bearer "));
        let result = json_body(&mut response);
        assert_eq!(result["code"], 401);
    }

    #[test]
    fn test_get_bad_auth() {
        let client = rocket_client();
        let mut response = client.get("/v1/broadcasts").header(Auth::Foo).dispatch();
        assert_eq!(response.status(), Status::Forbidden);
        let result = json_body(&mut response);
        assert_eq!(result["code"], 403);
    }

    #[test]
    fn test_put_get() {
        let client = rocket_client();
        let _ = client
            .put("/v1/broadcasts/foo/bar")
            .header(Auth::FooAlt)
            .body("v1")
            .dispatch();
        let _ = client
            .put("/v1/broadcasts/baz/quux")
            .header(Auth::Baz)
            .body("v0")
            .dispatch();
        let mut response = client.get("/v1/broadcasts").header(Auth::Reader).dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(
            json_body(&mut response),
            *json!({"code": 200, "broadcasts": {"baz/quux": "v0", "foo/bar": "v1"}})
        );
    }

    #[test]
    fn test_version() {
        let client = rocket_client();
        let mut response = client.get("/__version__").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let result = json_body(&mut response);
        assert_eq!(result["version"], "devel");
    }

    #[test]
    fn test_heartbeat() {
        let client = rocket_client();
        let mut response = client.get("/__heartbeat__").dispatch();
        assert_eq!(response.status(), Status::Ok);
        let result = json_body(&mut response);
        assert_eq!(result["code"], 200);
        assert_eq!(result["database"], "ok");
    }

    #[test]
    fn test_lbheartbeat() {
        let client = rocket_client();
        let mut response = client.get("/__lbheartbeat__").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert!(response.body().is_none());
    }
}
