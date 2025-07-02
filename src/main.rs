
use std::path;

use gt_tool::cli::Args;
use gt_tool::structs::release::{CreateReleaseOption, Release};

use clap::Parser;

use reqwest::header;
use reqwest::header::ACCEPT;

#[tokio::main]
async fn main() -> Result<(), gt_tool::Error> {
    let args = Args::parse();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.append(ACCEPT, header::HeaderValue::from_static("application/json"));

    // Gitea expects to see "token " for token auth.
    if let Ok(token) = std::env::var("RELEASE_KEY_GITEA") {
        let token = format!("token {token}");
        headers.append("Authorization", token.parse().unwrap());
    }
    let client = reqwest::Client::builder()
        .user_agent(format!(
            "gt-tools-agent-{}",
            env!("CARGO_PKG_VERSION")
        ))
        .default_headers(headers)
        .build()?;

    match args.command {
        gt_tool::cli::Commands::ListReleases => {
            let releases =
                gt_tool::api::release::list_releases(&client, &args.gitea_url, &args.repo).await?;
            for release in releases {
                println!("{}", release);
            }
        }
        gt_tool::cli::Commands::CreateRelease {
            name,
            body,
            draft,
            tag_name,
            target_commitish,
        } => {
            let submission = CreateReleaseOption {
                body,
                draft,
                name,
                prerelease: false,
                tag_name,
                target_commitish,
            };
            gt_tool::api::release::create_release(
                &client,
                &args.gitea_url,
                &args.repo,
                submission,
            )
            .await?;
        }
        gt_tool::cli::Commands::UploadRelease {
            tag_name,
            // create,
            files,
        } => {
            println!("Uploading files to a release!");
            println!("Release Tag: {tag_name}");
            // println!("Creating?: {create}");
            println!("Files...");
            for file in &files {
                println!("--- {file}");
            }
            // TODO: Pre-create the release, if it doesn't exist.

            // There's only Gitea APIs to get all releases, or one by-id.
            // Grab all, find the one that matches the input tag.
            // Scream if there are multiple matches.
            let release_candidates =
                gt_tool::api::release::list_releases(&client, &args.gitea_url, &args.repo).await?;

            if let Some(release) = match_release_by_tag(&tag_name, release_candidates) {
                for file in &files {
                    let path = path::Path::new(&file);
                    match path.try_exists() {
                        Ok(true) => continue,
                        Ok(false) => return Err(gt_tool::Error::NoSuchFile),
                        Err(e) => {
                            eprintln!("Uh oh! The file-exists check couldn't be done: {e}");
                            panic!("TODO: Deal with scenario where the file's existence cannot be checked (e.g.: no permission)");
                        },
                    }
                }
                for file in files {
                    let _attach_desc = gt_tool::api::release_attachment::create_release_attachment(
                        &client,
                        &args.gitea_url,
                        &args.repo,
                        release.id,
                        file,
                    )
                    .await?;
                }
            } else {
                println!("ERR: Couldn't find a release matching the tag \"{tag_name}\".");
                return Err(gt_tool::Error::NoSuchRelease);
            }
        }
    }

    Ok(())
}

// Util to scan a release list for a given tag. Returns Some(release) if found,
// None if not, and crashes the program if multiple are found.
//
// The Gitea webpage won't create multiple releases for a given tag, but the
// API might... And someone could always fiddle with the database directly.
// Until I find a guarantee of any particular behavior, I'm going to assume it
// *can* happen and crash when it does. Clearly it isn't supposed to, so I'm
// not going to meaningfully handle it, only prevent garbage from propagating.
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
