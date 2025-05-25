use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'o', long = "owner", env = "GTTOOL_OWNER")]
    pub owner: String,
    #[arg(short = 'r', long = "repo")]
    pub repo: Option<String>,

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
