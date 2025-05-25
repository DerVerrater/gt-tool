use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'u', long = "url", env = "GTTOOL_GITEA_URL")]
    pub gitea_url: String,
    #[arg(short = 'r', long = "repo", env = "GTTOOL_FQRN")]
    pub repo: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    ListReleases,
    CreateRelease {
        #[arg()]
        body: String,
        #[arg(short, long, default_value_t = false)]
        draft: bool,
        #[arg()]
        name: String,
        #[arg(short, long)]
        prerelease: bool,
        #[arg()]
        tag_name: String,
        #[arg()]
        target_commitish: String,
    },
}
