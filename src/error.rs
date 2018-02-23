/// ==== Error Handling ( see
/// https://boats.gitlab.io/blog/post/2017-11-30-failure-0-1-1/)
use std::result;

use failure;

pub type Result<T> = result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
enum MegaphoneError {
    #[fail(display = "{}: Invalid Version info (must be URL safe Base 64)", name)]
    InvalidVersionDataError {
        name: String,
    },

    #[fail(display = "{}: Version information not included in body of update", name)]
    MissingVersionDataError {
        name: String,
    },
}
