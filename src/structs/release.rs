use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub id: usize,
    pub tag_name: String,
    target_commitish: String,
    name: String,
    body: String,
    url: String,
    html_url: String,
    tarball_url: String,
    zipball_url: String,
    upload_url: String,
    draft: bool,
    prerelease: bool,
    created_at: String,
    published_at: String,
    author: Author,
}

impl Release {
    pub fn colorized(&self) -> String {
        let tag = "Tag:".green().bold();
        let name = "Name:".green();
        let published = "Published:".bright_green();
        let created = "Created:".green().dimmed();
        let author = "Author:".blue();
        let body = if !self.body.is_empty() {
            &self.body.white()
        } else {
            &String::from("(empty body)").dimmed()
        };

        format!(
            "{tag} {}
{name} {}
  {}
{published} {} ({created} {})
{author} {} ({})",
            self.tag_name.bold(),
            self.name,
            body,
            self.published_at,
            self.created_at.dimmed(),
            self.author.login,
            self.author.email,
        )
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Author {
    id: usize,
    login: String,
    login_name: String,
    source_id: usize,
    full_name: String,
    email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateReleaseOption {
    pub body: String,
    pub draft: bool,
    pub name: String,
    pub prerelease: bool,
    pub tag_name: String,
    pub target_commitish: String,
}

pub struct EditReleaseOption;
