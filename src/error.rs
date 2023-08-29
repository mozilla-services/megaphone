/// Error handling based on the failure crate
///
/// Only rocket's Handlers can render error responses w/ a contextual JSON
/// payload. So request guards should generally return VALIDATION_FAILED,
/// leaving error handling to the Handler (which in turn must take a Result of
/// request guards' fields).
///
/// HandlerErrors are rocket Responders (render their own error responses).
use std::fmt::{self, Display};
use std::result;

use backtrace::Backtrace;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::{self, response, Request, State};
use rocket_contrib::json;
use slog::{debug, warn};
use thiserror::Error;

use crate::logging::RequestLogger;

pub type Result<T> = result::Result<T, HandlerError>;

pub type HandlerResult<T> = result::Result<T, HandlerError>;

/// Signal a request guard failure, propagated up to the Handler to render an
/// error response
pub const VALIDATION_FAILED: Status = Status::InternalServerError;

#[derive(Debug)]
pub struct HandlerError {
    inner: HandlerErrorKind,
    pub backtrace: Backtrace,
}

#[derive(Debug, Error)]
pub enum HandlerErrorKind {
    /// 400 Bad Requests
    #[error("Invalid broadcasterID (must be URL safe base64, <= 64 characters)")]
    InvalidBroadcasterId,
    #[error("Invalid bchannelID (must be URL safe base64, <= 128 characters)")]
    InvalidBchannelId,

    #[error("Version information not included in body of update")]
    MissingVersionDataError,
    #[error("Invalid Version (must be ASCII, <= 200 characters)")]
    InvalidVersionDataError,

    /// 401 "Unauthorized" (unauthenticated)
    #[error("Missing authorization header")]
    MissingAuth,
    #[error("Invalid authorization header")]
    InvalidAuth,

    /// 403 Forbidden (unauthorized)
    #[error("Access denied to the requested resource")]
    Unauthorized,

    /// 404 Not Found
    #[error("Not Found")]
    NotFound,

    /// 500 Internal Server Errors
    #[error("Unexpected megaphone error: {0}")]
    InternalError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// 503 Service Unavailable
    #[error("A database error occurred: {0}")]
    DBError(#[from] diesel::result::Error),

    /// 503 Service Unavailable
    #[error("An error occurred while establishing a db connection: {0}")]
    DieselConnection(#[from] diesel::result::ConnectionError),

    /// 503 Service Unavailable
    #[error("A database pool error occurred: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),

    /// 503 Service Unavailable
    #[error("Error migrating the database: {0}")]
    Migration(#[from] diesel_migrations::RunMigrationsError),

    /// 413 Test Error
    #[error("Oh Noes!")]
    TestError,
}

impl HandlerErrorKind {
    /// Return a rocket response Status to be rendered for an error
    pub fn http_status(&self) -> Status {
        match self {
            HandlerErrorKind::MissingAuth | HandlerErrorKind::InvalidAuth => Status::Unauthorized,
            HandlerErrorKind::Unauthorized => Status::Forbidden,
            HandlerErrorKind::NotFound => Status::NotFound,
            HandlerErrorKind::InternalError(_) | HandlerErrorKind::IoError(_) => {
                Status::InternalServerError
            }
            HandlerErrorKind::DBError(_)
            | HandlerErrorKind::DieselConnection(_)
            | HandlerErrorKind::Migration(_)
            | HandlerErrorKind::Pool(_) => Status::ServiceUnavailable,
            _ => Status::BadRequest,
        }
    }

    /// Return a unique errno code
    pub fn errno(&self) -> i32 {
        match self {
            HandlerErrorKind::InvalidBroadcasterId => 100,
            HandlerErrorKind::InvalidBchannelId => 101,
            HandlerErrorKind::MissingVersionDataError => 102,
            HandlerErrorKind::InvalidVersionDataError => 103,

            HandlerErrorKind::MissingAuth => 120,
            HandlerErrorKind::InvalidAuth => 121,
            HandlerErrorKind::Unauthorized => 122,
            HandlerErrorKind::NotFound => 123,

            HandlerErrorKind::IoError(_) | HandlerErrorKind::InternalError(_) => 201,

            HandlerErrorKind::DBError(_)
            | HandlerErrorKind::DieselConnection(_)
            | HandlerErrorKind::Migration(_)
            | HandlerErrorKind::Pool(_) => 202,

            HandlerErrorKind::TestError => 413,
        }
    }
}

impl HandlerError {
    pub fn kind(&self) -> &HandlerErrorKind {
        &self.inner
    }

    /// Return an InternalError with the given error message
    pub fn internal(msg: String) -> Self {
        HandlerErrorKind::InternalError(msg).into()
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind(),)?;

        // Go down the chain of errors
        let mut error: &dyn std::error::Error = &self.inner;
        while let Some(source) = error.source() {
            write!(f, "\n\nCaused by: {source}")?;
            error = source;
        }

        Ok(())
    }
}

impl std::error::Error for HandlerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

// Forward From impls to HandlerError from HandlerErrorKind. Because From is
// reflexive, this impl also takes care of From<HandlerErrorKind>.
impl<T> From<T> for HandlerError
where
    HandlerErrorKind: From<T>,
{
    fn from(item: T) -> Self {
        HandlerError {
            inner: HandlerErrorKind::from(item),
            backtrace: Backtrace::new(),
        }
    }
}

/// Generate HTTP error responses for HandlerErrors
impl<'r> Responder<'r> for HandlerError {
    fn respond_to(self, request: &Request<'_>) -> response::Result<'r> {
        let status = self.kind().http_status();
        let errno = self.kind().errno();
        let log = RequestLogger::with_request(request).map_err(|_| Status::InternalServerError)?;
        let sentry_client = request
            .guard::<State<'_, Option<sentry::ClientInitGuard>>>()
            .succeeded();
        if sentry_client.is_some() {
            sentry::capture_event(sentry::event_from_error(&self));
        };
        match status {
            Status::Unauthorized | Status::Forbidden => {
                warn!(log, "{}", &self; "code" => status.code, "errno" => errno)
            }
            _ => debug!(log, "{}", &self; "code" => status.code, "errno" => errno),
        }

        let json = json!({
            "code": status.code,
            "errno": errno,
            "error": format!("{}", self)
        });
        let mut builder = Response::build_from(json.respond_to(request)?);
        if status == Status::Unauthorized {
            let environment = request
                .guard::<State<'_, rocket::config::Environment>>()
                .succeeded()
                .map(|state| *state)
                .unwrap_or(rocket::config::Environment::Development);
            builder.header(Header::new(
                "WWW-Authenticate",
                format!(r#"Bearer realm="{}""#, environment),
            ));
        }
        builder.status(status).ok()
    }
}
