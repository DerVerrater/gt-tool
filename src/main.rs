use gt_tools::structs::release::CreateReleaseOption;
use gt_tools::cli::Args;

use clap::Parser;

use reqwest::header;
use reqwest::header::ACCEPT;

#[tokio::main]
async fn main() -> Result<(), gt_tools::Error> {
    let args = Args::parse();
    
    let mut headers = reqwest::header::HeaderMap::new();
    headers.append(ACCEPT, header::HeaderValue::from_static("application/json"));
    
    // Gitea expects to see "token " for token auth.
    if let Ok(token) = std::env::var("RELEASE_KEY_GITEA") {
        let token = format!("token {token}");
        headers.append("Authorization", token.parse().unwrap());
    }
    let client = reqwest::Client::builder()
        .user_agent("gt-tools-test-agent")
        .default_headers(headers)
        .build()?;

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
            gt_tools::api::release::create_release(&client, &args.gitea_url, &args.repo, submission)
                .await?;
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
            gt_tools::api::release_attachment::create_release_attachment(
                &client,
                &args.gitea_url,
                &args.repo,
                52usize,
                files
            ).await?;
        }
    }

    Ok(())
}
