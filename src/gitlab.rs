use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

use crate::{repo::Repo, Password};

pub const PROVIDER: &'static str = "gitlab";

#[derive(Debug, Deserialize)]
struct GitlabFileResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct GitlabRepoResponse {
    id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GitlabRepo {
    project_id: String,
    path: String,
    token: Option<GitlabToken>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "token_type", content = "token")]
enum GitlabToken {
    Saved(String),
    FromEnv(String),
}

#[async_trait]
#[typetag::serde]
impl Repo for GitlabRepo {
    fn provider(&self) -> &'static str {
        PROVIDER
    }

    fn readable(&self) -> String {
        format!("gitlab.com/{}", &self.path)
    }

    async fn fetch_script(&self, path: &str, repo_ref: &str) -> Result<String> {
        let script_url = format!(
            "https://gitlab.com/api/v4/projects/{}/repository/files/{}?ref={}",
            self.project_id, path, repo_ref,
        );

        let req = reqwest::Client::new().get(script_url);
        let token = match &self.token {
            Some(GitlabToken::Saved(saved)) => Some(saved.clone()),
            Some(GitlabToken::FromEnv(var)) => Some(env::var(var)?),
            None => None,
        };

        let req = match token {
            Some(token) => req.header("PRIVATE-TOKEN", token),
            _ => req,
        };

        let resp = req.send().await?;
        if !resp.status().is_success() {
            bail!(
                "Got error response from gitlab: {}",
                resp.json::<serde_json::Value>().await?
            );
        }

        let resp = resp.json::<GitlabFileResponse>().await?;
        let decoded_content = base64::decode(resp.content)?;
        Ok(String::from_utf8(decoded_content)?)
    }
}

pub async fn fetch_project(uri: &Url, token: Password) -> Result<Box<dyn Repo>> {
    let without_leading_slash = uri.path().trim_start_matches('/');
    let encoded_uri = urlencoding::encode(without_leading_slash);
    let repo_url = format!("https://gitlab.com/api/v4/projects/{}", encoded_uri);
    let req = reqwest::Client::new().get(repo_url);

    let (req, token_to_save) = match token {
        Password::Saved(token) => (
            req.header("PRIVATE-TOKEN", token.clone()),
            Some(GitlabToken::Saved(token)),
        ),
        Password::FromEnv(var, token) => (
            req.header("PRIVATE-TOKEN", token),
            Some(GitlabToken::FromEnv(var)),
        ),
        _ => (req, None),
    };

    let resp = req.send().await?;
    if !resp.status().is_success() {
        bail!(
            "Got error response from gitlab: {}",
            resp.json::<serde_json::Value>().await?
        );
    }

    let resp = resp.json::<GitlabRepoResponse>().await?;
    let result = GitlabRepo {
        project_id: format!("{}", resp.id),
        token: token_to_save,
        path: without_leading_slash.to_owned(),
    };

    Ok(Box::new(result))
}
