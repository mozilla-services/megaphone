// Include for clippy error on `fn lbhearbeat` expansion
#![allow(clippy::let_unit_value)]

use std::env;
use std::io::Read;
use std::time::Instant;

use diesel::RunQueryDsl;
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
};
use crate::error::{HandlerError, HandlerErrorKind, HandlerResult, Result, VALIDATION_FAILED};
use crate::logging::{self, RequestLogger};
use crate::metrics::Metrics;
use crate::tags::Tags;

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
        if let Err(_e) = data.open().read_to_string(&mut value) {
            return Failure((
                VALIDATION_FAILED,
                HandlerErrorKind::MissingVersionDataError.into(),
            ));
        };
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

#[allow(clippy::too_many_arguments)]
/// Set a version for a broadcaster / bchannel
#[put("/v1/broadcasts/<broadcaster_id>/<bchannel_id>", data = "<version>")]
fn broadcast(
    conn: HandlerResult<db::Conn>,
    log: RequestLogger,
    broadcaster: HandlerResult<Broadcaster>,
    broadcaster_id: String,
    bchannel_id: String,
    version: HandlerResult<VersionInput>,
    metrics: Metrics,
    base_tags: Tags,
) -> HandlerResult<status::Custom<JsonValue>> {
    let conn = conn?;

    if broadcaster_id.len() > 64 || !URLSAFE_B64_RE.is_match(&broadcaster_id) {
        Err(HandlerErrorKind::InvalidBroadcasterId)?
    }
    if bchannel_id.len() > 128 || !URLSAFE_B64_RE.is_match(&bchannel_id) {
        Err(HandlerErrorKind::InvalidBchannelId)?
    }

    let mut tags = base_tags;
    let version = version?.value;

    tags.tags
        .insert("broadcaster".to_owned(), broadcaster_id.clone());
    tags.tags
        .insert("channel_id".to_owned(), bchannel_id.clone());
    tags.tags.insert("version".to_owned(), version.clone());
    metrics.incr_with_tags("broadcast.cmd.update", Some(tags.clone()));

    let start = Instant::now();
    let created = broadcaster?.broadcast_new_version(&conn, &bchannel_id, &version)?;
    metrics.timer_with_tags(
        "broadcast.update",
        (Instant::now() - start).as_millis() as u64,
        Some(tags),
    );
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
    metrics: Metrics,
) -> HandlerResult<JsonValue> {
    metrics.incr("broadcast.cmd.dump");
    let conn = conn?;
    let start = Instant::now();
    let broadcasts = reader?.read_broadcasts(&conn)?;
    metrics.timer_with_tags(
        "broadcast.dump",
        (Instant::now() - start).as_millis() as u64,
        None,
    );

    Ok(json!({
        "code": 200,
        "broadcasts": broadcasts
    }))
}

#[get("/v1/err")]
fn log_check(
    _conn: HandlerResult<db::Conn>,
    log: RequestLogger,
) -> HandlerResult<content::Json<&'static str>> {
    info!(log, "Oh my!");
    error!(log, "Oh dear!");
    // Attempt to force a message through.
    // sentry::capture_message("Oh bother!", sentry::protocol::Level::Info);
    // panic!("Oh noes!")
    Err(HandlerErrorKind::TestError.into())
}

#[get("/__version__")]
fn version() -> content::Json<&'static str> {
    content::Json(include_str!("../version.json"))
}

#[get("/__heartbeat__")]
fn heartbeat(conn: HandlerResult<db::Conn>, log: RequestLogger) -> status::Custom<JsonValue> {
    let result = conn.and_then(|conn| {
        Ok(diesel::sql_query("SELECT 1")
            .execute(&*conn)
            .map_err(HandlerErrorKind::DBError)?)
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

pub fn get_sentry(config: &rocket::config::Config) -> Option<sentry::ClientInitGuard> {
    let opts = sentry::ClientOptions {
        // debug: true,
        ..Default::default()
    };
    if let Ok(sentry_dsn) = config.get_string("sentry_dsn") {
        Some(sentry::init((sentry_dsn, opts)))
    } else {
        // Check the global env to see if we need to connect to sentry.
        if env::var("SENTRY_DSN").is_ok() {
            Some(sentry::init(opts))
        } else {
            None
        }
    }
}

pub fn rocket() -> Result<Rocket> {
    // RocketConfig::init basically
    let rconfig = RocketConfig::read().unwrap_or_else(|_| {
        let path = env::current_dir()
            .unwrap()
            .join(format!(".{}.{}", "default", "Rocket.toml"));
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
    let sentry_client = get_sentry(rocket.config());
    let logger = logging::init_logging(rocket.config(), &sentry_client)?;
    let tags = Tags::init(rocket.config())?;
    let metrics = Metrics::init(rocket.config(), &sentry_client)?;
    info!(logger, "Starting up");
    db::run_embedded_migrations(rocket.config())?;
    Ok(rocket
        .manage(pool)
        .manage(authenticator)
        .manage(environment)
        .manage(logger)
        .manage(metrics)
        .manage(tags)
        .manage(sentry_client)
        .mount(
            "/",
            routes![
                broadcast,
                get_broadcasts,
                version,
                heartbeat,
                lbheartbeat,
                log_check
            ],
        )
        .register(catchers![not_found]))
}

#[cfg(test)]
mod test {
    use crate::auth::test::to_table;
    use rocket::config::{Config, Environment, RocketConfig, Value as RValue};
    use rocket::http::{Header, Status};
    use rocket::local::Client;
    use rocket::response::Response;
    use rocket_contrib::json;
    use serde_json::{self, Value};

    use super::setup_rocket;

    /// Test auth headers
    enum Auth {
        Foo,
        FooAlt,
        Baz,
        Reader,
    }

    impl From<Auth> for Header<'static> {
        fn from(auth: Auth) -> Header<'static> {
            let token = match auth {
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
            .expect("ROCKET_DATABASE_URL undefined")
            .to_owned();
        let config = Config::build(Environment::Development)
            .extra("database_url", RValue::String(database_url))
            .extra("database_pool_max_size", 1)
            .extra("database_use_test_transactions", true)
            .extra("json_logging", false)
            .extra(
                "broadcaster_auth",
                to_table(
                    [
                        "foo=feedfacedeadbeef,deadbeeffacefeed",
                        "baz=baada555deadbeef",
                    ]
                    .to_vec(),
                ),
            )
            .extra(
                "reader_auth",
                to_table(["reader=00000000deadbeef"].to_vec()),
            )
            .unwrap();
        dbg!(&config);

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
