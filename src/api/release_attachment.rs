use std::{fs, path};

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

    let path = path::Path::new(&file);
    match path.try_exists() {
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
            .map_err( crate::Error::from)?;
        return Ok(attachment_desc);
    } else if response.status().is_client_error() {
        let mesg = crate::decode_client_error(response).await?;
        return Err(crate::Error::ApiErrorMessage(mesg));
    }
    panic!("Reached end of release_attachment without matching a return path");
}
pub fn edit_release_attachment() {}
pub fn delete_release_attachment() {}
