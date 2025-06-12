use serde::{Deserialize, Serialize};

pub mod release;
pub mod repo;

#[derive(Debug, Deserialize, Serialize)]
pub struct Attachment {
    id: usize,
    name: String,
    size: i64,
    download_count: i64,
    created_at: String, // TODO: Date-time struct
    uuid: String,
    browser_download_url: String,
}
