use crate::{
    github::{self, GithubRepo},
    gitlab::{self, GitlabRepo},
};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    convert::TryInto,
    io::{self, Write},
    process::Command,
};

const SHELL_NAME: &'static str = "rem";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenericRepo {
    pub provider: String,
    pub uri: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub password_env: Option<String>,
}

#[async_trait]
pub trait Repo {
    fn id() -> &'static str;
    async fn get(&self, path: &str, repo_ref: &str) -> Result<String>;
}

impl GenericRepo {
    pub async fn get_contents(self, script_name: &str, repo_ref: &str) -> Result<String> {
        match self.provider.as_str() {
            gitlab::PROVIDER => {
                let gitlab_repo: GitlabRepo = self.try_into()?;
                Ok(gitlab_repo.get(script_name, repo_ref).await?)
            }
            github::PROVIDER => {
                let github_repo: GithubRepo = self.try_into()?;
                Ok(github_repo.get(script_name, repo_ref).await?)
            }
            _ => bail!("Unknown provider: `{}`", &self.provider),
        }
    }
}

pub fn run_script(script: &str, script_args: Vec<&str>) -> Result<()> {
    let mut cmd = Command::new("bash");
    let mut args = vec!["-c", script, SHELL_NAME];
    args.extend_from_slice(&script_args);

    cmd.args(&args);
    let _child = cmd.spawn()?;

    Ok(())
}

pub fn import_script(script: &str) -> Result<()> {
    io::stdout().write_all(script.as_bytes())?;
    Ok(())
}
