use serde::{Deserialize, Serialize};

pub mod cli;
pub mod api;
pub mod structs;

#[derive(Debug, Serialize)]
pub struct CreateReleaseOption {
    pub body: String,
    pub draft: bool,
    pub name: String,
    pub prerelease: bool,
    pub tag_name: String,
    pub target_commitish: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    message: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CreateResult {
    Success(structs::release::Release),
    ErrWithMessage(ApiError),
    Empty,
}
