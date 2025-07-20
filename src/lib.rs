use serde::{Deserialize, Serialize};

pub mod api;
pub mod cli;
pub mod config;
pub mod structs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    message: String,
    url: String,
}

pub(crate) async fn decode_client_error(response: reqwest::Response) -> Result<ApiError> {
    response
        .json::<ApiError>()
        .await
        .map_err(crate::Error::WrappedReqwestErr)
}

#[derive(Debug)]
pub enum Error {
    Placeholder,     // TODO: Enumerate error modes
    MissingGiteaUrl, // the gitea URL wasn't specified on the CLI, env, or config file.
    MissingRepoFRQN, // either the owner, repo, or both weren't specified in the loaded PartialConfig
    WrappedConfigErr(config::Error),
    WrappedReqwestErr(reqwest::Error),
    MissingAuthToken,
    NoSuchFile, // for release attachment 'file exists' pre-check.
    NoSuchRelease,
    ApiErrorMessage(ApiError),
}

impl From<reqwest::Error> for crate::Error {
    fn from(value: reqwest::Error) -> Self {
        Self::WrappedReqwestErr(value)
    }
}

impl From<crate::config::Error> for crate::Error {
    fn from(value: crate::config::Error) -> Self {
        Self::WrappedConfigErr(value)
    }
}

type Result<T> = core::result::Result<T, Error>;
