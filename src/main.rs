use gt_tools::structs::release::CreateReleaseOption;
use gt_tools::{structs::release::Release, cli::Args};
use reqwest::multipart::Part;

use std::collections::HashMap;
use std::{
    env,
    fs
};

use clap::Parser;

use reqwest::Error;
use reqwest::header::{ACCEPT, USER_AGENT};

const API_RELEASE_FRONT: &'static str = "/api/v1/repos/";
const API_RELEASE_BACK: &'static str = "/releases";

#[tokio::main]
async fn main() -> Result<(), gt_tools::Error> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    match args.command {
        gt_tools::cli::Commands::ListReleases => {
            let releases = gt_tools::api::release::list_releases(
                &client,
                &args.gitea_url,
                &args.repo
            ).await?;
            for release in releases {
                println!("{:?}", release);
            }
        }
        gt_tools::cli::Commands::CreateRelease {
            name,
            body,
            draft,
            prerelease,
            tag_name,
            target_commitish,
        } => {
            let submission = CreateReleaseOption {
                body,
                draft,
                name,
                prerelease,
                tag_name,
                target_commitish,
            };
            do_create_release(&client, &args.gitea_url, &args.repo, submission)
                .await
                .map_err(reqwest_to_gttool)?;
        }
        gt_tools::cli::Commands::UploadRelease {
            tag_name,
            create,
            files
        } => {
            println!("Uploading files to a release!");
            println!("Release Tag: {tag_name}");
            println!("Creating?: {create}");
            println!("Files...");
            for file in &files {
                println!("--- {file}");
            }
            // TODO: Pre-create the release, if it doesn't exist.
            // TODO: Find an existing release and use it's ID, if it does
            do_upload_release(
                &client,
                &args.gitea_url,
                &args.repo,
                52usize,
                files
            ).await.map_err(reqwest_to_gttool)?;
        }
    }

    Ok(())
}

fn reqwest_to_gttool(err: reqwest::Error) -> gt_tools::Error {
    gt_tools::Error::WrappedReqwestErr(err)
}

#[must_use]
async fn do_create_release(
    client: &reqwest::Client,
    gitea_url: &str,
    repo: &str,
    submission: CreateReleaseOption,
) -> Result<(), Error> {
    let token = env::var("RELEASE_KEY_GITEA").expect(
        "You must set the RELEASE_KEY_GITEA environment variable so the Gitea API can be used.",
    );
    let request_url = format!(
        "{gitea_url}{front}{repo}{back}",
        front = API_RELEASE_FRONT,
        back = API_RELEASE_BACK
    );
    let response = client
        .post(request_url)
        .header(USER_AGENT, "gt-tools-test-agent")
        .header(ACCEPT, "application/json")
        .header("Authorization", format!("token {}", token))
        .json(&submission)
        .send()
        .await?;

    println!("HTTP Response: {}", response.status());
    let result: gt_tools::CreateResult = response.json().await?;
    println!("{:?}", result);
    Ok(())
}

async fn do_upload_release(
    client: &reqwest::Client,
    gitea_url: &str,
    repo: &str,
    release_id: usize,
    files: Vec<String>,
) -> Result<(), Error> {
    let token = env::var("RELEASE_KEY_GITEA").expect(
        "You must set the RELEASE_KEY_GITEA environment variable so the Gitea API can be used.",
    );
    let request_url = format!(
        "{gitea_url}{front}{repo}{back}/{id}/assets",
        front = API_RELEASE_FRONT,
        back = API_RELEASE_BACK,
        id = release_id
    );
    // Setup a "partial request". I'll need to loop over the files, so I'll
    // clone this a bunch of times for each one
    let partial_request = client
        .post(request_url)
        .header(USER_AGENT, "gt-tools-test-agent")
        .header(ACCEPT, "application/json")
        .header("Authorization", format!("token {}", token));
    
    // Ensure all files exists before starting the uploads
    for file in &files {
        assert!(fs::exists(file)
            .expect(&format!("Can't check existence of {file}"))
        );
    }

    for file in files {
        if let Some(request) = partial_request.try_clone() {
            println!("Uploading file {}", &file);
            let data = reqwest::multipart::Part::stream(fs::read(&file).unwrap())
                .file_name("attachment")
                .mime_str("text/plain")?;

            let form= reqwest::multipart::Form::new()
                .part("attachment", data);

            let request = request
                .multipart(form)
                .query(&[("name", file.split("/").last())])
                .send()
                .await?;
        } else {
            panic!("Failed to clone the RequestBuilder during file upload loop.");
        }
    }
    Ok(())
}
