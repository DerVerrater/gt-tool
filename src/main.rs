use gt_tools::CreateReleaseOption;
use gt_tools::{ReleaseInfo, cli::Args};

use std::env;

use clap::Parser;

use reqwest::Error;
use reqwest::header::{ACCEPT, USER_AGENT};

const API_HOSTNAME: &'static str = "http://localhost:3000";
const API_RELEASE_FRONT: &'static str = "/api/v1/repos/";
const API_RELEASE_BACK: &'static str = "/releases";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    match args.command {
        gt_tools::cli::Commands::ListReleases => {
            let releases = do_list_releases(&client, "robert", "rcalc").await?;
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
            do_create_release(&client, submission).await?;
        }
    }

    Ok(())
}

async fn do_list_releases(
    client: &reqwest::Client,
    owner: &str,
    repo: &str,
) -> Result<Vec<ReleaseInfo>, Error> {
    let request_url = format!(
        "{hostname}{front}{owner}/{repo}{back}",
        hostname = API_HOSTNAME,
        front = API_RELEASE_FRONT,
        back = API_RELEASE_BACK
    );
    let response = client
        .get(request_url)
        .header(USER_AGENT, "gt-tools-test-agent")
        .header(ACCEPT, "application/json")
        .send()
        .await?;
    // TODO: Handle case with no releases.
    // afaict: Serde tries to unpack an empty list, can't decide what struct it's unpacking,
    // and emits an error. Desired behavior: empty Vec.
    let body_text: Vec<ReleaseInfo> = response.json().await?;
    return Ok(body_text);
}

#[must_use]
async fn do_create_release(
    client: &reqwest::Client,
    submission: CreateReleaseOption,
) -> Result<(), Error> {
    let token = env::var("RELEASE_KEY_GITEA").expect(
        "You must set the RELEASE_KEY_GITEA environment variable so the Gitea API can be used.",
    );
    let request_url = format!(
        "{hostname}{front}{owner}/{repo}{back}",
        hostname = API_HOSTNAME,
        front = API_RELEASE_FRONT,
        owner = "robert",
        repo = "rcalc",
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
