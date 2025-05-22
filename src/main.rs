use gt_tools::ReleaseInfo;

use reqwest::Error;
use reqwest::header::{
    USER_AGENT,
    ACCEPT
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let request_url = format!(
        "http:/localhost:3000/api/v1/repos/{owner}/{repo}/releases",
        owner = "robert",
        repo = "rcalc",
    );
    let client = reqwest::Client::new();
    let response = client
        .get(request_url)
        .header(USER_AGENT, "gt-tools-test-agent")
        .header(ACCEPT, "application/json")
        .send()
        .await?;
    let body_text: Vec<ReleaseInfo> = response.json().await?;

    println!("{:?}", body_text);
    Ok(())
}
