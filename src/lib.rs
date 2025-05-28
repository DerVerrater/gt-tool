use serde::{Deserialize, Serialize};

pub mod cli;
pub mod api;
pub mod structs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    message: String,
    url: String,
}

#[derive(Debug)]
pub enum Error {
    Placeholder, // TODO: Enumerate error modes
    WrappedReqwestErr(reqwest::Error)
}
type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CreateResult {
    Success(structs::release::Release),
    ErrWithMessage(ApiError),
    Empty,
}
