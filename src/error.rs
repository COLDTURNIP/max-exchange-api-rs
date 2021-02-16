//! Error/Result types definition and handling.

use serde::Deserialize;
use std::convert::From;
use std::fmt;
use thiserror::Error;

/// [`std::result::Result`] with [`enum@Error`]
pub type Result<T> = std::result::Result<T, Error>;

/// All possible errors used in this crate.
#[derive(Debug, Error)]
pub enum Error {
    /// Error message returned by API server. See "Response Format" in [official document](https://max.maicoin.com/documents/api_v2).
    #[error("API error code {0}: {1}")]
    RestApi(u64, String),

    /// I/O error while reading body from HTTP response.
    // http_types::Error wraps anyhow::Error, but it does not implement str::err::Error in Rust 2018
    #[error("Unable read response")]
    ReadResponse(Box<anyhow::Error>),

    /// Invalid content in websocket request/response body.
    #[error("Invalid value: {0}")]
    WsInvalidValue(String),

    /// Errors during parsing websocket messages.
    #[error(transparent)]
    WsApiParse(serde_json::Error),
}

#[derive(Deserialize, Debug)]
struct ApiErrorDetail {
    code: u64,
    message: String,
}

impl fmt::Display for ApiErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MaiCoin MAX Error {}: {}", self.code, self.message)
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct ApiErrorWrapper {
    error: ApiErrorDetail,
}

impl From<ApiErrorWrapper> for Error {
    fn from(err: ApiErrorWrapper) -> Self {
        Error::RestApi(err.error.code, err.error.message)
    }
}
