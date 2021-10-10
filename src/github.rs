use crate::{repo::Repo, Password};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Debug, Deserialize)]
struct GithubFileResponse {
    download_url: String,
}

pub const PROVIDER: &'static str = "github";

#[derive(Serialize, Deserialize, Clone)]
pub struct GithubRepo {
    project_id: String,
    auth: Option<GithubAuth>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "password_type", content = "password")]
enum GithubPassword {
    Saved(String),
    FromEnv(String),
}

#[derive(Serialize, Deserialize, Clone)]
struct GithubAuth {
    username: String,
    password: GithubPassword,
}

#[async_trait]
#[typetag::serde]
impl Repo for GithubRepo {
    fn provider(&self) -> &'static str {
        PROVIDER
    }

    fn readable(&self) -> String {
        format!("github.com/{}", &self.project_id)
    }

    fn box_clone(&self) -> Box<dyn Repo> {
        Box::new(self.clone())
    }

    async fn fetch_script(&self, path: &str, repo_ref: &str) -> Result<String> {
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

pub async fn fetch_project(
    uri: &Url,
    username: Option<String>,
    password: Password,
) -> Result<Box<dyn Repo>> {
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

    Ok(Box::new(result))
}
