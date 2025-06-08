use serde::{Deserialize, Serialize};

use crate::{
    ApiError, Result,
    structs::{
        self,
        release::{CreateReleaseOption, Release},
    },
};

pub fn get_release(id: u64) -> Result<Release> {
    todo!();
}
pub fn get_latest_release() -> Result<Release> {
    todo!();
}

pub async fn list_releases(
    client: &reqwest::Client,
    gitea_url: &str,
    repo: &str,
) -> Result<Vec<Release>> {
    let request_url = format!("{gitea_url}/api/v1/repos/{repo}/releases/");
    let req = client.get(request_url).send().await;
    let response = req.map_err(|reqwest_err| crate::Error::WrappedReqwestErr(reqwest_err))?;
    if response.status().is_success() {
        let release_list = response
            .json::<Vec<Release>>()
            .await
            .map_err(|reqwest_err| {
                // Convert reqwest errors to my own
                // TODO: Create all error variants (see lib.rs)
                crate::Error::WrappedReqwestErr(reqwest_err)
            })?;
        return Ok(release_list);
    } else if response.status().is_client_error() {
        let mesg = response
            .json::<ApiError>()
            .await
            .map_err(|reqwest_err| {
                crate::Error::WrappedReqwestErr(reqwest_err)
            })?;
        return Err(crate::Error::ApiErrorMessage(mesg));
    }
    panic!("Reached end of list_releases without matching a return pathway.");
}

pub async fn create_release(
    client: &reqwest::Client,
    gitea_url: &str,
    repo: &str,
    submission: CreateReleaseOption,
) -> Result<Release> {
    let request_url = format!("{gitea_url}/api/v1/repos/{repo}/releases");
    let response = client
        .post(request_url)
        .json(&submission)
        .send()
        .await
        .map_err(|e| crate::Error::from(e))?;
    if response.status().is_success() {
        let new_release = response
            .json::<Release>()
            .await
            .map_err(|e| crate::Error::from(e))?;
        return Ok(new_release);
    } else if response.status().is_client_error() {
        let mesg = response
            .json::<ApiError>()
            .await
            .map_err(|reqwest_err| {
                crate::Error::WrappedReqwestErr(reqwest_err)
            })?;
        return Err(crate::Error::ApiErrorMessage(mesg))
    }
    panic!("Reached end of create_release without matching a return path");
}
pub fn edit_release(id: u64) -> Result<Release> {
    todo!();
}
pub fn delete_release(id: u64) -> Result<()> {
    todo!();
}
