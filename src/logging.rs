/// Logging via slog
///
/// Provides a RequestLogger with moz log fields per request
use std::io;
use std::ops::Deref;

use lazy_static::lazy_static;
use mozsvc_common::{aws::get_ec2_instance_id, get_hostname};
use rocket::{
    config::ConfigError,
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest},
    Config, Request, State,
};
use sentry_slog::SentryDrain;
use slog::{self, slog_o, Drain};
use slog_derive::KV;
use slog_mozlog_json::MozLogJson;

use crate::error::{HandlerError, HandlerResult};

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
    pub fn from_request(request: &Request<'_>) -> MozLogFields {
        let headers = request.headers();
        MozLogFields {
            method: request.method().as_str(),
            path: request.uri().to_string(),
            agent: headers.get_one("User-Agent").map(str::to_owned),
            remote: headers
                .get_one("X-Forwarded-For")
                .map(str::to_owned)
                .or_else(|| request.remote().map(|addr| addr.ip().to_string())),
        }
    }
}

pub struct RequestLogger(slog::Logger);

impl RequestLogger {
    pub fn with_request(request: &Request<'_>) -> HandlerResult<RequestLogger> {
        let logger =
            request
                .guard::<State<'_, RequestLogger>>()
                .success_or(HandlerError::internal(
                    "Internal error: No managed RequestLogger".to_owned(),
                ))?;
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
    type Error = HandlerError;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        RequestLogger::with_request(request).into_outcome(Status::InternalServerError)
    }
}

pub fn init_logging(
    config: &Config,
    sentry: &Option<sentry::ClientInitGuard>,
) -> HandlerResult<RequestLogger> {
    let json_logging = match config.get_bool("json_logging") {
        Ok(json_logging) => json_logging,
        Err(ConfigError::Missing(_)) => true,
        Err(e) => {
            return Err(HandlerError::internal(format!(
                "Invalid ROCKET_JSON_LOGGING: {}",
                e
            )))
        }
    };

    let async_drain = if json_logging {
        let hostname = match get_ec2_instance_id() {
            Some(v) => v.to_owned(),
            None => match get_hostname() {
                Ok(v) => v.to_string_lossy().to_string(),
                Err(e) => {
                    return Err(HandlerError::internal(format!(
                        "Could not drain async: {}",
                        e
                    )))
                }
            },
        };

        let drain = MozLogJson::new(io::stdout())
            .logger_name(LOGGER_NAME.to_owned())
            .msg_type(MSG_TYPE.to_owned())
            .hostname(hostname)
            .build()
            .fuse();
        slog_async::Async::new(drain).build().fuse()
    } else {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        slog_async::Async::new(drain).build().fuse()
    };

    /* By default, only `panic!()` messages are captured and sent to sentry.
       Setting a drain doesn't always capture other errors.

      The `.mapper()` method is not defined for any version prior to 0.20, so we
      can't use that to force things by capturing the event there. Likewise, for
      whatever reason sentry > 0.19 doesn't create a working transport layer to
      communicate with itself.

      Specifying the client also doesn't seem to want to connect up, at least,
      while it doesn't report an error, it also doesn't send anything through.

    */
    let logger = if sentry.is_some() {
        dbg!("Connecting to sentry...");
        slog::Logger::root(SentryDrain::new(async_drain).fuse(), slog_o!())
    } else {
        slog::Logger::root(async_drain, slog_o!())
    };
    Ok(RequestLogger(logger))
}
