use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::{
    convert::{From, TryFrom},
    env,
};
use url::Url;

use crate::{
    repo::{GenericRepo, Repo},
    Password,
};

#[derive(Debug, Deserialize)]
struct GithubFileResponse {
    download_url: String,
}

pub const PROVIDER: &'static str = "github";

pub struct GithubRepo {
    project_id: String,
    auth: Option<GithubAuth>,
}

enum GithubPassword {
    Saved(String),
    FromEnv(String),
}

struct GithubAuth {
    username: String,
    password: GithubPassword,
}

#[async_trait]
impl Repo for GithubRepo {
    fn id() -> &'static str {
        PROVIDER
    }

    async fn get(&self, path: &str, repo_ref: &str) -> Result<String> {
        let script_url = format!(
            "https://api.github.com/repos/{}/contents/{}?ref={}",
            self.project_id, path, repo_ref,
        );

        let req = reqwest::Client::new()
            .get(script_url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "rem-bash");

        let auth = match &self.auth {
            Some(auth) => {
                let password = match &auth.password {
                    GithubPassword::Saved(saved) => saved.to_string(),
                    GithubPassword::FromEnv(var) => env::var(var)?,
                };

                Some((auth.username.clone(), password))
            }
            None => None,
        };

        let req = match auth {
            Some((username, password)) => req.basic_auth(username, Some(password)),
            _ => req,
        };

        let resp = req.send().await?;
        if !resp.status().is_success() {
            bail!(
                "Got error response from gitlab: {}",
                resp.json::<serde_json::Value>().await?
            );
        }

        let resp = resp.json::<GithubFileResponse>().await?;
        let content = reqwest::Client::new()
            .get(&resp.download_url)
            .header("User-Agent", "rem-bash")
            .send()
            .await?
            .text()
            .await?;

        Ok(content)
    }
}

impl From<GithubRepo> for GenericRepo {
    fn from(github_repo: GithubRepo) -> Self {
        let (password, password_env) = match github_repo.auth.as_ref().map(|it| &it.password) {
            Some(GithubPassword::Saved(saved)) => (Some(saved), None),
            Some(GithubPassword::FromEnv(var)) => (None, Some(var)),
            _ => (None, None),
        };

        GenericRepo {
            provider: GithubRepo::id().to_string(),
            uri: github_repo.project_id,
            username: github_repo.auth.as_ref().map(|it| it.username.to_owned()),
            password: password.map(|it| it.to_owned()),
            password_env: password_env.map(|it| it.to_owned()),
        }
    }
}

impl TryFrom<GenericRepo> for GithubRepo {
    type Error = anyhow::Error;
    fn try_from(repo: GenericRepo) -> Result<Self> {
        let auth = if let Some(username) = repo.username {
            let password = match (repo.password, repo.password_env) {
                (Some(_), Some(_)) => {
                    bail!("Github repo cannot have both passsword and password_env")
                }
                (Some(saved), None) => GithubPassword::Saved(saved),
                (None, Some(var)) => GithubPassword::FromEnv(var),
                _ => bail!("Github repo must have password if there is a username"),
            };

            Some(GithubAuth { username, password })
        } else {
            None
        };

        Ok(Self {
            project_id: repo.uri,
            auth,
        })
    }
}

pub async fn fetch_project(
    uri: &Url,
    username: Option<String>,
    password: Password,
) -> Result<GenericRepo> {
    let without_leading_slash = uri.path().trim_start_matches('/');
    let repo_url = format!("https://api.github.com/repos/{}", without_leading_slash);
    let req = reqwest::Client::new()
        .get(repo_url)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "rem-bash");

    if username.is_some() && password == Password::None {
        bail!("Github repo must have password if a username is used");
    }

    let (req, password_to_save) = match password {
        Password::Saved(password) => (
            req.basic_auth(username.clone().unwrap(), Some(password.clone())),
            Some(GithubPassword::Saved(password)),
        ),
        Password::FromEnv(var, password) => (
            req.basic_auth(username.clone().unwrap(), Some(password.clone())),
            Some(GithubPassword::FromEnv(var)),
        ),
        _ => (req, None),
    };

    let resp = req.send().await?;
    if !resp.status().is_success() {
        bail!("Got error response from github: {}", resp.text().await?);
    }

    let auth = username.map(|username| GithubAuth {
        username: username.to_string(),
        password: password_to_save.unwrap(),
    });

    let result = GithubRepo {
        project_id: without_leading_slash.to_string(),
        auth,
    };

    Ok(result.into())
}
