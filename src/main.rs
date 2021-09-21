use std::{io::{self, Write}, env, process::Command};
use anyhow::Result;
use serde::Deserialize;
use clap::{crate_version, clap_app};

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
    let app = clap_app!(rem =>
        (version: crate_version!())
        (author: "Hilmar Wiegand <me@hwgnd.de>")
        (about: "Remote bash script execution and library import")
        (@setting DeriveDisplayOrder)
        (@setting ColoredHelp)
        (@setting SubcommandRequiredElseHelp)

        (@subcommand repo =>
            (about: "Read and modify locally saved repositories")
            (@setting DeriveDisplayOrder)
            (@setting ColoredHelp)
            (@setting SubcommandRequiredElseHelp)

            (@subcommand list =>
                (about: "Lists all locally saved repositories")
                (@setting ColoredHelp)
            )

            (@subcommand add =>
                (about: "Adds a repository to the local repository list")
                (@setting ColoredHelp)
                (@arg NAME: +required "Local alias for the repository to add")
                (@arg URI: +required "URI of the repository to add")
                (@arg username: -u --username [USERNAME] "Username for the repository (if required)")
                (@arg password: -p --password [PASSWORD] "Password or token for the repository (if required)")
                (@arg password_stdin: -i --("password-stdin") "Reads the password or token from stdin")
            )

            (@subcommand check =>
                (about: "Checks whether a repository is accessible and prints out details about the repository")
                (@setting ColoredHelp)
                (@arg NAME: +required "Local alias for the repository to add")
            )

            (@subcommand remove =>
                (about: "Removes a repository from the local repository list")
                (@setting ColoredHelp)
                (@arg NAME: +required "Local alias for the repository to remove")
            )
        )

        (@subcommand import =>
            (about: "Imports a script and prints it to stdout")
            (@setting ColoredHelp)
            (@arg SCRIPT: +required "Script identifier in the format `<repo>:<script_path>`")
        )

        (@subcommand run =>
            (about: "Runs a script using the locally installed bash shell")
            (@setting ColoredHelp)
            (@arg SCRIPT: +required "Script identifier in the format `<repo>:<script_path>`")
        )
    );

    let args = app.get_matches();
    match args.subcommand() {
        ("repo", Some(args)) => match args.subcommand() {
            ("list", Some(_)) => cmd_repo_list(),
            ("add", Some(args)) => cmd_repo_add(),
            ("check", Some(args)) => cmd_repo_check(),
            ("remove", Some(args)) => cmd_repo_remove(),
            _ => unreachable!(),
        },
        ("import", Some(args)) => cmd_import().await?,
        ("run", Some(args)) => cmd_run().await?,
        _ => unreachable!(),
    }

    Ok(())
}

fn cmd_repo_list() {}

fn cmd_repo_add() {}

fn cmd_repo_check() {}

fn cmd_repo_remove() {}

async fn cmd_import() -> Result<()> {
    let gitlab_token = env::var("GITLAB_TOKEN").expect("No GitLab token found");
    let script_url = "https://gitlab.com/api/v4/projects/0000/repository/files/helloworld%2Ebash?ref=main";
    let resp = reqwest::Client::new()
        .get(script_url)
        .header("PRIVATE-TOKEN", gitlab_token)
        .send().await?
        .json::<GitlabFileResponse>().await?;

    let decoded_content = base64::decode(resp.content)?;
    let decoded_content = String::from_utf8(decoded_content)?;
    import_script(&decoded_content)
}

async fn cmd_run() -> Result<()> {
    let gitlab_token = env::var("GITLAB_TOKEN").expect("No GitLab token found");
    let script_url = "https://gitlab.com/api/v4/projects/0000/repository/files/helloworld%2Ebash?ref=main";
    let resp = reqwest::Client::new()
        .get(script_url)
        .header("PRIVATE-TOKEN", gitlab_token)
        .send().await?
        .json::<GitlabFileResponse>().await?;

    let decoded_content = base64::decode(resp.content)?;
    let decoded_content = String::from_utf8(decoded_content)?;
    run_script(&decoded_content, vec![])
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
