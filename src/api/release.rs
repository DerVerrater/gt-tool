
use crate::{
    structs::release::{CreateReleaseOption, Release}, CreateResult, Result
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

pub async fn create_release(
    client: &reqwest::Client,
    gitea_url: &str,
    repo: &str,
    submission: CreateReleaseOption
) -> Result<Release> {
    let request_url = format!("{gitea_url}/api/v1/repos/{repo}/releases");
    let req = client
        .post(request_url)
        .json(&submission)
        .send()
        .await
        .map_err(|e| crate::Error::from(e))?;
    let new_release = req
        .json::<CreateResult>()
        .await
        .map_err(|e| crate::Error::from(e))?;
    match new_release {
        CreateResult::Success(release) => Ok(release),
        CreateResult::ErrWithMessage(api_error) => {
            if api_error.message == "token is required" {
                Err(crate::Error::MissingAuthToken)
            } else {
                Err(crate::Error::ApiErrorMessage(api_error))
            }
        },
        CreateResult::Empty => panic!("How can we have 200 OK and no release info? No. Crash"),
    }
}
pub fn edit_release(id: u64) -> Result<Release> { todo!(); }
pub fn delete_release(id: u64) -> Result<()> { todo!(); }

