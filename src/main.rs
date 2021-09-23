use anyhow::{anyhow, bail, Context, Result};
use clap::{AppSettings, Clap};
use std::{
    env,
    io::{self, Read},
};
use url::Url;

use crate::config::save_config;

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
    /// Script identifier in the format `<repo>[@version]:<script_path>`
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
    let mut config = config::load_config().await?;
    // println!("config: {:#?}", config);

    match Opts::parse().command {
        Command::Repo(repo) => match repo.command {
            RepoCommand::List => println!("Listing repos"),
            RepoCommand::Add {
                name,
                uri,
                username,
                password,
                password_env,
                password_stdin,
            } => {
                if config.repo.contains_key(&name) {
                    bail!("A repository with the name `{}` already exists", &name);
                }

                let password_for_parse = match (password, password_env, password_stdin) {
                    (Some(pass), _, _) => Some(pass),
                    (_, Some(var), _) => Some(env::var(var)?),
                    (_, _, true) => {
                        let mut buf = String::new();
                        io::stdin().read_to_string(&mut buf)?;
                        Some(buf)
                    }
                    _ => None,
                };

                let repo = get_repo(&uri, username, password_for_parse).await?;
                config.repo.insert(name, repo);
                save_config(&config).await?;
            }
            RepoCommand::Check { .. } => {}
            RepoCommand::Remove { .. } => {}
        },
        Command::Run(_script) => {}
        Command::Import(_script) => {}
    };

    Ok(())
}

async fn get_repo(
    uri: &str,
    _username: Option<String>,
    password: Option<String>,
) -> Result<repo::GenericRepo> {
    let mut maybe_parsed: Option<Url> = None;

    // Check if we've been given a raw gitlab or github url without scheme
    if uri.starts_with("gitlab.com") || uri.starts_with("github.com") {
        let with_scheme = format!("https://{}", uri);
        maybe_parsed = Some(Url::parse(&with_scheme)?);
    }

    // Try parsing the url manually otherwise
    let mut parsed = match maybe_parsed {
        Some(parsed) => parsed,
        None => Url::parse(uri)?,
    };

    if parsed.cannot_be_a_base() {
        bail!("Repo URI was not recognized");
    }

    // Enforce https
    let _ = parsed.set_scheme("https");

    match parsed.host_str() {
        Some("gitlab.com") => Ok(gitlab::fetch_project(&parsed, password).await?),
        Some(_) => bail!("No provider recognized for passed URI"),
        None => bail!("No host on passed URI"),
    }
}
