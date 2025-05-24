use serde::{Deserialize, Serialize};

pub mod cli;

pub fn module_echo() {
    println!("hello from lib.rs!");
}

/// A struct matching a Gitea "Release" entry
#[derive(Deserialize, Debug, Serialize)]
pub struct ReleaseInfo {
    id: usize,
    tag_name: String,
    target_commitish: String,
    name: String,
    body: String,
    url: String,
    html_url: String,
    tarball_url: String,
    zipball_url: String,
    upload_url: String,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: String,
    author: Author,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Author {
    id: usize,
    login: String,
    login_name: String,
    source_id: usize,
    full_name: String,
    email: String,
}

#[derive(Serialize, Debug)]
pub struct CreateReleaseOption {
    body: String,
    draft: bool,
    name: String,
    prerelease: bool,
    tag_name: String,
    target_commitish: String,
}

impl CreateReleaseOption {
    pub fn new(name: String) -> Self {
        Self{
            body: String::from("hard-coded test body"),
            draft: false,
            name,
            prerelease: true,
            tag_name: String::from("big-goof"),
            target_commitish: String::from("548ceecc7528901a7b4376091b42e410d950affc")
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    message: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CreateResult {
    Success(ReleaseInfo),
    ErrWithMessage(ApiError),
    Empty
}
