use std::fs;

use crate::structs::Attachment;

pub fn check_release_match_repo() {}
pub fn get_release_attachment() {}
pub fn list_release_attachments() {
    todo!();
}
pub async fn create_release_attachment(
    client: &reqwest::Client,
    gitea_url: &str,
    repo: &str,
    release_id: usize,
    file: String,
) -> crate::Result<Attachment> {
    let request_url = format!("{gitea_url}/api/v1/repos/{repo}/releases/{release_id}/assets");

    match fs::exists(&file) {
        Ok(true) => (),
        Ok(false) => return Err(crate::Error::NoSuchFile),
        Err(e) => {
            eprintln!("Uh oh! The file-exists check couldn't be done: {e}");
            panic!("TODO: Deal with scenario where the file's existence cannot be checked (e.g.: no permission)");
        },
    }

    println!("Uploading file {}", &file);
    let data = reqwest::multipart::Part::stream(fs::read(&file).unwrap())
        .file_name("attachment")
        .mime_str("text/plain")?;

    let form = reqwest::multipart::Form::new().part("attachment", data);

    let response = client
        .post(&request_url)
        .multipart(form)
        .query(&[("name", file.split("/").last())])
        .send()
        .await?;
    if response.status().is_success() {
        // TODO: create a struct Attachment and return it to the caller.
        let attachment_desc = response
            .json::<Attachment>()
            .await
            .map_err(|e| crate::Error::from(e))?;
        return Ok(attachment_desc);
    } else if response.status().is_client_error() {
        let mesg = crate::decode_client_error(response).await?;
        return Err(crate::Error::ApiErrorMessage(mesg));
    }
    panic!("Reached end of release_attachment without matching a return path");
}
pub fn edit_release_attachment() {}
pub fn delete_release_attachment() {}

#[cfg(test)]
mod tests {
    use reqwest::header::{self, ACCEPT};

    use crate::structs::release::Release;


    #[tokio::test]
    async fn attach_file_exists() {
        let conf = TestConfig::new();
        let release_candidates =
            crate::api::release::list_releases(
                &conf.client,
                &conf.server,
                &conf.repo
            )
            .await
            .expect("Failed to get releases. Pre-conditions unmet, aborting test!");

        let release = match_release_by_tag(&conf.release_tag, release_candidates)
            .expect("Failed to select matching release. Pre-conditions unmet, aborting test!");
        
        let api_result = super::create_release_attachment(
            &conf.client,
            &conf.server,
            &conf.repo,
            release.id,
            vec![String::from("Cargo.toml")],
        )
        .await;
    }

    #[tokio::test]
    async fn attach_file_missing() {
        let conf = TestConfig::new();
        let release_candidates =
            crate::api::release::list_releases(
                &conf.client,
                &conf.server,
                &conf.repo
            )
            .await
            .expect("Failed to get releases. Pre-conditions unmet, aborting test!");

        let release = match_release_by_tag(&conf.release_tag, release_candidates)
            .expect("Failed to select matching release. Pre-conditions unmet, aborting test!");
        
        let api_result = super::create_release_attachment(
            &conf.client,
            &conf.server,
            &conf.repo,
            release.id,
            vec![String::from("./this-file-doesnt-exist")],
        )
        .await;
        let api_err = api_result.expect_err("Received Ok(()) after uploading non-existent file. That's nonsense, the API Function is wrong.");
        match api_err {
            crate::Error::Placeholder=>panic!("Received dummy response from the API function. Finish implementing it, stupid"),
            crate::Error::WrappedReqwestErr(error)=>panic!("Received a reqwest::Error from the API function: {error}"),
            crate::Error::MissingAuthToken=>unreachable!("Missing auth token... in a unit test that already panics without the auth token..."),
            crate::Error::NoSuchFile=>(),
            crate::Error::ApiErrorMessage(api_error)=>panic!("Received an error message from the API: {api_error:?}"),
            crate::Error::NoSuchRelease => todo!(),
        }
    }

    struct TestConfig {
        server: String,
        repo: String,
        release_tag: String,
        client: reqwest::Client,
    }

    impl TestConfig {
        fn new() -> Self {
            let server = std::env::var("TEST_GITEA_SERVER")
                .expect("Must set server address in env var \"TEST_GITEA_SERVER\"");
            let repo = std::env::var("TEST_GITEA_REPO")
                .expect("Must set <user>/<repo> name in env var \"TEST_GITEA_REPO\"");
            let token = format!(
                "token {}",
                std::env::var("TEST_GITEA_KEY")
                    .expect("Must set the API token in env var \"TEST_GITEA_KEY\"")
            );
            let release_tag = std::env::var("TEST_GITEA_RELEASE_TAG")
                .expect("Must set the target release tag in env var \"TEST_GITEA_RELEASE_TAG\"");

            let mut headers = reqwest::header::HeaderMap::new();
            headers.append(ACCEPT, header::HeaderValue::from_static("application/json"));
            headers.append("Authorization", token.parse().unwrap());

            let client = reqwest::Client::builder()
                .user_agent(format!(
                    "gt-tools-autotest-agent{}",
                    env!("CARGO_PKG_VERSION")
                ))
                .default_headers(headers)
                .build()
                .expect("Failed to build reqwest::Client.");

            return Self {
                server,
                repo,
                release_tag,
                client
            };
        }
    }

    // Testing utils
    fn match_release_by_tag(tag: &String, releases: Vec<Release>) -> Option<Release> {
        let mut release: Option<Release> = None;
        for rel in releases {
            if rel.tag_name == *tag {
                // Only store the value if one hasn't been stored already
                if let Some(first_release) = &release {
                    // if there was already a match, begin the error diagnostic creation.
                    let first_id = first_release.id;
                    let second_id = rel.id;
                    assert!(
                        first_id != second_id,
                        "FAILURE: Found the same release ID twice while scanning for duplicate tags. How did we get the same one twice?"
                    );
                    eprintln!("ERROR: Two releases have been found for the tag \"{tag}\".");
                    eprintln!("ERROR:    first ID: {first_id}");
                    eprintln!("ERROR:    second ID: {second_id}");
                    panic!("ERROR: Nonsense detected, I'm bailing out!");
                } else {
                    // else, store our first (and hopefully only) match
                    release = Some(rel);
                }
            }
        }
        return release;
    }
}