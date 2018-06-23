/// Error handling based on the failure crate
///
/// Only rocket's Handlers can render error responses w/ a contextual JSON
/// payload. So request guards should generally return VALIDATION_FAILED,
/// leaving error handling to the Handler (which in turn must take a Result of
/// request guards' fields).
///
/// HandlerErrors are rocket Responders (render their own error responses).
use std::fmt;
use std::result;

use failure::{Backtrace, Context, Error, Fail};
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::{self, response, Request, State};
use rocket_contrib::Json;

use logging::RequestLogger;

pub type Result<T> = result::Result<T, Error>;

pub type HandlerResult<T> = result::Result<T, HandlerError>;

/// Signal a request guard failure, propagated up to the Handler to render an
/// error response
pub const VALIDATION_FAILED: Status = Status::InternalServerError;

#[derive(Debug)]
pub struct HandlerError {
    inner: Context<HandlerErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum HandlerErrorKind {
    /// 400 Bad Requests
    #[fail(display = "Invalid broadcasterID (must be URL safe base64, <= 64 characters)")]
    InvalidBroadcasterId,
    #[fail(display = "Invalid bchannelID (must be URL safe base64, <= 128 characters)")]
    InvalidBchannelId,

    #[fail(display = "Version information not included in body of update")]
    MissingVersionDataError,
    #[fail(display = "Invalid Version (must be ASCII, <= 200 characters)")]
    InvalidVersionDataError,

    /// 401 "Unauthorized" (unauthenticated)
    #[fail(display = "Missing authorization header")]
    MissingAuth,
    #[fail(display = "Invalid authorization header")]
    InvalidAuth,

    /// 403 Forbidden (unauthorized)
    #[fail(display = "Access denied to the requested resource")]
    Unauthorized,

    /// 404 Not Found
    #[fail(display = "Not Found")]
    NotFound,

    /// 500 Internal Server Errors
    #[fail(display = "Unexpected rocket error: {:?}", _0)]
    RocketError(rocket::Error), // rocket::Error isn't a std Error (so no #[cause])
    #[fail(display = "Unexpected megaphone error")]
    InternalError,

    /// 503 Service Unavailable
    #[fail(display = "A database error occurred")]
    DBError,
}

impl HandlerErrorKind {
    /// Return a rocket response Status to be rendered for an error
    pub fn http_status(&self) -> Status {
        match self {
            HandlerErrorKind::MissingAuth | HandlerErrorKind::InvalidAuth => Status::Unauthorized,
            HandlerErrorKind::Unauthorized => Status::Forbidden,
            HandlerErrorKind::NotFound => Status::NotFound,
            HandlerErrorKind::RocketError(_) | HandlerErrorKind::InternalError => {
                Status::InternalServerError
            }
            HandlerErrorKind::DBError => Status::ServiceUnavailable,
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

            HandlerErrorKind::RocketError(_) => 200,
            HandlerErrorKind::InternalError => 201,
            HandlerErrorKind::DBError => 202,
        }
    }
}

impl HandlerError {
    pub fn kind(&self) -> &HandlerErrorKind {
        self.inner.get_context()
    }
}

impl Fail for HandlerError {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl From<HandlerErrorKind> for HandlerError {
    fn from(kind: HandlerErrorKind) -> HandlerError {
        Context::new(kind).into()
    }
}

impl From<Context<HandlerErrorKind>> for HandlerError {
    fn from(inner: Context<HandlerErrorKind>) -> HandlerError {
        HandlerError { inner }
    }
}

/// Generate HTTP error responses for HandlerErrors
impl<'r> Responder<'r> for HandlerError {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        let status = self.kind().http_status();
        let errno = self.kind().errno();
        let log = RequestLogger::with_request(request).map_err(|_| Status::InternalServerError)?;
        match status {
            Status::Unauthorized | Status::Forbidden => {
                slog_warn!(log, "{}", &self; "code" => status.code, "errno" => errno)
            }
            _ => slog_debug!(log, "{}", &self; "code" => status.code, "errno" => errno),
        }

        let json = Json(json!({
            "code": status.code,
            "errno": errno,
            "error": format!("{}", self)
        }));
        let mut builder = Response::build_from(json.respond_to(request)?);
        if status == Status::Unauthorized {
            let environment = request
                .guard::<State<rocket::config::Environment>>()
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
