use anyhow::Result;
use async_trait::async_trait;
use async_process::{Command, ExitStatus};
use std::fmt::Debug;
use tokio::io::{self, AsyncWriteExt};

const SHELL_NAME: &'static str = "rem";

#[async_trait]
#[typetag::serde(tag = "provider")]
pub trait Repo {
    fn provider(&self) -> &'static str;
    fn readable(&self) -> String;
    fn box_clone(&self) -> Box<dyn Repo>;
    async fn fetch_script(&self, path: &str, repo_ref: &str, fresh: bool) -> Result<String>;
}

impl Debug for Box<dyn Repo> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.provider(), self.readable())
    }
}

pub async fn run_script(script: &str, script_args: Vec<&str>) -> Result<ExitStatus> {
    let mut cmd = Command::new("bash");
    let mut args = vec!["-c", script, SHELL_NAME];
    args.extend_from_slice(&script_args);

    cmd.args(&args);
    let mut child = cmd.spawn()?;
    Ok(child.status().await?)
}

pub async fn import_script(script: &str) -> Result<()> {
    io::stdout().write_all(script.as_bytes()).await?;
    Ok(())
}
