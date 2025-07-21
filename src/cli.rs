use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'u', long = "url", env = "GTTOOL_GITEA_URL")]
    pub gitea_url: Option<String>,
    #[arg(short = 'o', long = "owner", env = "GTTOOL_OWNER")]
    pub owner: Option<String>,
    #[arg(short = 'r', long = "repo", env = "GTTOOL_REPO")]
    pub repo: Option<String>,
    #[arg(
        short = 'p',
        long = "project",
        help = "Path to project (relative or absolute). Used to select configuration."
    )]
    pub project: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    ListReleases,
    CreateRelease {
        #[arg()]
        body: String,
        #[arg(short, long)]
        draft: bool,
        #[arg()]
        name: String,
        #[arg()]
        tag_name: String,
        #[arg()]
        target_commitish: String,
    },
    UploadRelease {
        #[arg()]
        tag_name: String,
        // TODO: implement create-and-upload as a single command invocation
        // #[arg(short, long)]
        // create: bool,
        #[arg()]
        files: Vec<String>,
    },
}
