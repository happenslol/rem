use std::{io::{self, Write}, env, process::Command};
use anyhow::Result;
use serde::Deserialize;

const SHELL_NAME: &'static str = "rem";

#[derive(Debug, Deserialize)]
struct GitlabFileResponse {
    file_name: String,
    file_path: String,
    size: u64,
    content: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let gitlab_token = env::var("GITLAB_TOKEN").expect("No GitLab token found");
    let script_url = "https://gitlab.com/api/v4/projects/29355703/repository/files/helloworld%2Ebash?ref=main";
    let resp = reqwest::Client::new()
        .get(script_url)
        .header("PRIVATE-TOKEN", gitlab_token)
        .send().await?
        .json::<GitlabFileResponse>().await?;

    let decoded_content = base64::decode(resp.content)?;
    let decoded_content = String::from_utf8(decoded_content)?;
    println!("got content: {}", decoded_content);

    Ok(())
}

fn run_script(script: &str, script_args: Vec<&str>) -> Result<()> {
    let mut cmd = Command::new("bash");
    let mut args = vec!["-c", script, SHELL_NAME];
    args.extend_from_slice(&script_args);

    cmd.args(&args);
    let _child = cmd.spawn()?;

    Ok(())
}

fn import_script(script: &str) -> Result<()> {
    io::stdout().write_all(script.as_bytes())?;

    Ok(())
}
