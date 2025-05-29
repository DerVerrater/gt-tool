
use crate::{
    structs::release::Release, Result
};

pub fn get_release(id: u64) -> Result<Release> { todo!(); }
pub fn get_latest_release() -> Result<Release> { todo!(); }

pub async fn list_releases(
    client: &reqwest::Client,
    gitea_url: &str,
    repo: &str,
) -> Result<Vec<Release>> {
    let request_url = format!("{gitea_url}/api/v1/repos/{repo}/releases/");
    let req = client
        .get(request_url)
        .send()
        .await;
    let response = req
        .map_err(|reqwest_err| {
            crate::Error::WrappedReqwestErr(reqwest_err)
        })?;
    let release_list = response
        .json::<Vec<Release>>()
        .await
        .map_err(|reqwest_err| {
            // Convert reqwest errors to my own
            // TODO: Create all error variants (see lib.rs)
            crate::Error::WrappedReqwestErr(reqwest_err)
        })?;
    return Ok(release_list);
}

pub fn create_release() -> Result<Release> { todo!(); }
pub fn edit_release(id: u64) -> Result<Release> { todo!(); }
pub fn delete_release(id: u64) -> Result<()> { todo!(); }

