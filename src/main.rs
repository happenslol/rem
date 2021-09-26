use anyhow::{anyhow, bail, Result};
use clap::{AppSettings, Clap};
use std::{
    env,
    io::{self, Read},
};
use url::Url;

use crate::{config::save_config, repo::Repo as _};

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
    /// Script identifier in the format `<repo>[@ref]:<script_path>`
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
    #[clap(alias = "ls")]
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
    #[clap(alias = "rm")]
    Remove {
        /// Local alias for the repository to remove
        name: String,
    },
}

pub enum Password {
    Saved(String),
    FromEnv(String, String),
    None,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut config = config::load_config().await?;

    match Opts::parse().command {
        Command::Repo(repo) => match repo.command {
            RepoCommand::List => {
                println!("Saved repositories:");
                for (k, v) in config.repo {
                    println!("    {} ({}:{})", k, v.provider, v.uri);
                }
            }
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
                    (Some(pass), _, _) => Password::Saved(pass),
                    (_, Some(var), _) => Password::FromEnv(var.clone(), env::var(var)?),
                    (_, _, true) => {
                        let mut buf = String::new();
                        io::stdin().read_to_string(&mut buf)?;
                        Password::Saved(buf)
                    }
                    _ => Password::None,
                };

                let repo = get_repo(&uri, username, password_for_parse).await?;
                config.repo.insert(name.clone(), repo);
                println!("Repo `{}` was successfully added", &name);
                save_config(&config).await?;
            }
            RepoCommand::Check { .. } => unimplemented!(),
            RepoCommand::Remove { name } => {
                if !config.repo.contains_key(&name) {
                    bail!("Repo `{}` was not found", &name);
                }

                config.repo.remove(&name);
                save_config(&config).await?;
                println!("Repo `{}` was removed", &name);
            }
        },
        Command::Run(script) => {
            let contents = get_script_contents(&config, &script.script).await?;
            repo::run_script(&contents, vec![])?;
        }
        Command::Import(script) => {
            let contents = get_script_contents(&config, &script.script).await?;
            repo::import_script(&contents)?;
        }
    };

    Ok(())
}

async fn get_script_contents(config: &config::Config, script: &str) -> Result<String> {
    let parts = script.split(":").collect::<Vec<&str>>();
    if parts.len() != 2 {
        bail!("Script must be in the format `<repo>[@ref]:<script_path>`");
    }

    let repo_name = parts[0].to_string();
    let script_name = parts[1].to_string();

    let repo_parts = script.split("@").collect::<Vec<&str>>();
    let (repo_name, repo_ref) = match repo_parts.len() {
        1 => (repo_name, "HEAD".to_string()),
        2 => (parts[0].to_string(), parts[1].to_string()),
        _ => bail!("Invalid repo: `{}`", repo_name),
    };

    let generic_repo = config
        .repo
        .get(&repo_name)
        .ok_or(anyhow!("Repo `{}` was not found", &repo_name))?
        .clone();

    // TODO: Check script name for lib/bash extensions

    let repo = generic_repo.into_repo()?;
    let script_contents = repo.get(&script_name, &repo_ref).await?;

    Ok(script_contents)
}

async fn get_repo(
    uri: &str,
    _username: Option<String>,
    password: Password,
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
