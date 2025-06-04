use std::fs;

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
    files: Vec<String>,
) -> crate::Result<()> {
    let request_url = format!("{gitea_url}/api/v1/repos/{repo}/releases/{release_id}/assets");

    // Ensure all files exists before starting the uploads
    for file in &files {
        if let Err(e) = fs::exists(file) {
            return Err(crate::Error::NoSuchFile);
        }
    }

    for file in files {
        println!("Uploading file {}", &file);
        let data = reqwest::multipart::Part::stream(fs::read(&file).unwrap())
            .file_name("attachment")
            .mime_str("text/plain")?;

        let form = reqwest::multipart::Form::new().part("attachment", data);

        let request = client
            .post(&request_url)
            .multipart(form)
            .query(&[("name", file.split("/").last())])
            .send()
            .await?;
    }
    Ok(())
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

    #[test]
    fn attach_file_missing() {
        todo!();
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