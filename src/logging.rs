/// Logging via slog
///
/// Provides a RequestLogger with moz log fields per request
use std::io;
use std::ops::Deref;

use failure::{self, err_msg};
use mozsvc_common::{aws::get_ec2_instance_id, get_hostname};
use rocket::{
    config::ConfigError,
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest},
    Config, Request, State,
};
use slog::{self, Drain};
use slog_async;
use slog_derive::KV;
use slog_mozlog_json::MozLogJson;
use slog_term;

use error::Result;

lazy_static! {
    static ref LOGGER_NAME: String =
        format!("{}-{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    static ref MSG_TYPE: String = format!("{}:log", env!("CARGO_PKG_NAME"));
}

#[derive(Clone, KV)]
struct MozLogFields {
    method: &'static str,
    path: String,
    remote: Option<String>,
    agent: Option<String>,
}

impl MozLogFields {
    pub fn from_request(request: &Request) -> MozLogFields {
        let headers = request.headers();
        MozLogFields {
            method: request.method().as_str(),
            path: request.uri().to_string(),
            agent: headers.get_one("User-Agent").map(&str::to_owned),
            remote: headers
                .get_one("X-Forwarded-For")
                .map(&str::to_owned)
                .or_else(|| request.remote().map(|addr| addr.ip().to_string())),
        }
    }
}

pub struct RequestLogger(slog::Logger);

impl RequestLogger {
    pub fn with_request(request: &Request) -> Result<RequestLogger> {
        let logger = request
            .guard::<State<RequestLogger>>()
            .success_or(err_msg("Internal error: No managed RequestLogger"))?;
        Ok(RequestLogger(
            logger.new(slog_o!(MozLogFields::from_request(request))),
        ))
    }
}

impl Deref for RequestLogger {
    type Target = slog::Logger;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for RequestLogger {
    type Error = failure::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, failure::Error> {
        RequestLogger::with_request(request).into_outcome(Status::InternalServerError)
    }
}

pub fn init_logging(config: &Config) -> Result<RequestLogger> {
    let json_logging = match config.get_bool("json_logging") {
        Ok(json_logging) => json_logging,
        Err(ConfigError::Missing(_)) => true,
        Err(e) => Err(format_err!("Invalid ROCKET_JSON_LOGGING: {}", e))?,
    };

    let logger = if json_logging {
        let hostname = get_ec2_instance_id()
            .map(&str::to_owned)
            .or_else(get_hostname)
            .ok_or_else(|| err_msg("Couldn't get_hostname"))?;

        let drain = MozLogJson::new(io::stdout())
            .logger_name(LOGGER_NAME.to_owned())
            .msg_type(MSG_TYPE.to_owned())
            .hostname(hostname)
            .build()
            .fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        slog::Logger::root(drain, slog_o!())
    } else {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        slog::Logger::root(drain, slog_o!())
    };
    Ok(RequestLogger(logger))
}
