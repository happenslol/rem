use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, Write},
    process::Command,
};

const SHELL_NAME: &'static str = "rem";

#[derive(Debug, Deserialize, Serialize)]
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
    async fn get(&self, path: &str) -> Result<String>;
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
