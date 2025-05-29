use std::fs;


pub fn check_release_match_repo(){}
pub fn get_release_attachment(){}
pub fn list_release_attachments(){
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
    
        let form= reqwest::multipart::Form::new()
            .part("attachment", data);

        let request = client
            .post(&request_url)
            .multipart(form)
            .query(&[("name", file.split("/").last())])
            .send()
            .await?;
    }
    Ok(())
}
pub fn edit_release_attachment(){}
pub fn delete_release_attachment(){}

