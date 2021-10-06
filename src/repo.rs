use anyhow::Result;
use async_trait::async_trait;
use std::{
    fmt::Debug,
    io::{self, Write},
    process::Command,
};

const SHELL_NAME: &'static str = "rem";

#[async_trait]
#[typetag::serde(tag = "provider")]
pub trait Repo {
    fn provider(&self) -> &'static str;
    fn readable(&self) -> String;
    async fn fetch_script(&self, path: &str, repo_ref: &str) -> Result<String>;
}

impl Debug for Box<dyn Repo> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} | {}", self.provider(), self.readable())
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
