use clap::{
    Parser,
    Subcommand
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short = 'o', long = "owner")]
    repo_owner: Option<String>,
    #[arg(short = 'n', long = "repo_name")]
    repo_name: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand, Debug)]
enum Commands {
    ListReleases{
        #[arg(short, long)]
        list: bool,
    },
}
