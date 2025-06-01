use serde::{Deserialize, Serialize};

pub mod api;
pub mod cli;
pub mod structs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    message: String,
    url: String,
}

#[derive(Debug)]
pub enum Error {
    Placeholder, // TODO: Enumerate error modes
    WrappedReqwestErr(reqwest::Error),
    MissingAuthToken,
    NoSuchFile, // for release attachment 'file exists' pre-check.
    ApiErrorMessage(ApiError),
}

impl From<reqwest::Error> for crate::Error {
    fn from(value: reqwest::Error) -> Self {
        Self::WrappedReqwestErr(value)
    }
}

type Result<T> = core::result::Result<T, Error>;
