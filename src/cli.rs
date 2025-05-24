use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'o', long = "owner")]
    pub repo_owner: Option<String>,
    #[arg(short = 'n', long = "repo_name")]
    pub repo_name: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    ListReleases,
    CreateRelease {
        #[arg()]
        name: String,
    },
}
