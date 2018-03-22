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
use rocket::{self, response, Request};
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket_contrib::Json;

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
    /// 401 Unauthorized
    #[fail(display = "Unauthorized: {}", _0)]
    Unauthorized(String),
    /// 404 Not Found
    #[fail(display = "Not Found")]
    NotFound,
    #[fail(display = "A database error occurred")]
    DBError,
    #[fail(display = "Version information not included in body of update")]
    MissingVersionDataError,
    #[fail(display = "Invalid Version info (must be URL safe Base 64)")]
    InvalidVersionDataError,
    #[fail(display = "Unexpected rocket error: {:?}", _0)]
    RocketError(rocket::Error), // rocket::Error isn't a std Error (so no #[cause])
}

impl HandlerErrorKind {
    /// Return a rocket response Status to be rendered for an error
    pub fn http_status(&self) -> Status {
        match *self {
            HandlerErrorKind::Unauthorized(..) => Status::Unauthorized,
            HandlerErrorKind::NotFound => Status::NotFound,
            HandlerErrorKind::DBError => Status::ServiceUnavailable,
            _ => Status::BadRequest,
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
        HandlerError { inner: inner }
    }
}

/// Generate HTTP error responses for HandlerErrors
impl<'r> Responder<'r> for HandlerError {
    fn respond_to(self, request: &Request) -> response::Result<'r> {
        let status = self.kind().http_status();
        let json = Json(json!({
            "code": status.code,
            "error": format!("{}", self)
        }));
        // XXX: logging
        Response::build_from(json.respond_to(request)?)
            .status(status)
            .ok()
    }
}
