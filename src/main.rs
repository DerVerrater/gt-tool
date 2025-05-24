use gt_tools::CreateReleaseOption;
use gt_tools::{ReleaseInfo, cli::Args};
use serde::{Deserialize, Serialize};

use std::env;
use std::io::Read;

use clap::Parser;

use reqwest::Error;
use reqwest::header::{ACCEPT, USER_AGENT};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let request_url = format!(
        "http://localhost:3000/api/v1/repos/{owner}/{repo}/releases",
        owner = "robert",
        repo = "rcalc",
    );
    let client = reqwest::Client::new();
    match args.command {
        gt_tools::cli::Commands::ListReleases => {
            let response = client
                .get(request_url)
                .header(USER_AGENT, "gt-tools-test-agent")
                .header(ACCEPT, "application/json")
                .send()
                .await?;
            let body_text: Vec<ReleaseInfo> = response.json().await?;

            println!("{:?}", body_text);
        }
        gt_tools::cli::Commands::CreateRelease { name } => {
            let submission = CreateReleaseOption {
                body: String::from("hard-coded test body"),
                draft: false,
                name,
                prerelease: true,
                tag_name: String::from("big-goof"),
                target_commitish: String::from("548ceecc7528901a7b4376091b42e410d950affc"),
            };
            do_create_release(&client, &request_url, submission).await?;
        }
    }

    Ok(())
}

#[must_use]
async fn do_create_release(
    client: &reqwest::Client,
    endpoint: &str,
    submission: CreateReleaseOption,
) -> Result<(), Error> {
    let token = env::var("RELEASE_KEY_GITEA").expect(
        "You must set the RELEASE_KEY_GITEA environment variable so the Gitea API can be used.",
    );
    let response = client
        .post(endpoint)
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
