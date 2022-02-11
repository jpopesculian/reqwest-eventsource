use core::fmt;
use eventsource_stream::EventStreamError;
use nom::error::Error as NomError;
use reqwest::header::HeaderValue;
use reqwest::Error as ReqwestError;
use reqwest::StatusCode;
use std::string::FromUtf8Error;

/// Error raised when a [`RequestBuilder`] cannot be cloned. See [`RequestBuilder::try_clone`] for
/// more information
#[derive(Debug, Clone, Copy)]
pub struct CannotCloneRequestError;

pub enum Error {
    Utf8(FromUtf8Error),
    Parser(NomError<String>),
    Transport(ReqwestError),
    InvalidContentType(HeaderValue),
    InvalidStatusCode(StatusCode),
    StreamEnded,
}

impl From<EventStreamError<ReqwestError>> for Error {
    fn from(err: EventStreamError<ReqwestError>) -> Self {
        match err {
            EventStreamError::Utf8(err) => Self::Utf8(err),
            EventStreamError::Parser(err) => Self::Parser(err),
            EventStreamError::Transport(err) => Self::Transport(err),
        }
    }
}

impl fmt::Display for CannotCloneRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("expected a cloneable request")
    }
}

impl std::error::Error for CannotCloneRequestError {}
