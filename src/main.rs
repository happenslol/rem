use anyhow::Result;
use clap::{AppSettings, Clap};

mod config;
mod gitlab;
mod repo;

#[derive(Clap, Debug)]
#[clap(author, about, version)]
#[clap(global_setting = AppSettings::ColoredHelp)]
#[clap(setting = AppSettings::DeriveDisplayOrder)]
#[clap(setting = AppSettings::SubcommandRequiredElseHelp)]
struct Opts {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Clap, Debug)]
enum Command {
    /// Read and modify locally saved repositories
    Repo(Repo),
    /// Run a script using the locally installed bash shell
    Run(Script),
    /// Import a script and prints it to stdout
    Import(Script),
}

#[derive(Clap, Debug)]
struct Script {
    /// Script identifier in the format `<repo>:<script_path>`
    script: String,
}

#[derive(Clap, Debug)]
struct Repo {
    #[clap(subcommand)]
    command: RepoCommand,
}

#[derive(Clap, Debug)]
enum RepoCommand {
    /// List all locally saved repositories
    List,
    /// Add a repository to the local repository list
    Add {
        /// Local alias for the repository to add
        name: String,
        /// URI of the repository to add
        uri: String,

        /// Username for the repository (if required)
        #[clap(long, short)]
        username: Option<String>,
        /// Password or token for the repository (if required)
        #[clap(long, short)]
        password: Option<String>,
        /// Reads the password from the given environment variable when the repo is used
        #[clap(long)]
        password_env: Option<String>,
        /// Reads the password or token from stdin
        #[clap(long)]
        password_stdin: bool,
    },
    /// Check whether a repository is accessible and prints out details about the repository
    Check {
        /// Local alias for the repository to check
        name: String,
    },
    /// Remove a repository from the local repository list
    Remove {
        /// Local alias for the repository to remove
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let config = config::load_config().await?;

    println!("config: {:#?}", config);
    println!("opts: {:#?}", opts);

    Ok(())
}
