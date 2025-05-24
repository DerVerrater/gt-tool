use serde::{Deserialize, Serialize};

pub mod cli;

pub fn module_echo() {
    println!("hello from lib.rs!");
}

/// A struct matching a Gitea "Release" entry
#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
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
            draft: true,
            name,
            prerelease: true,
            tag_name: String::from("hard-coded-test"),
            target_commitish: String::from("3171c892480c46976106fa465d04fdb7e734dd53")
        }
    }
}
