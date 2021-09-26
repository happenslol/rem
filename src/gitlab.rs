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

pub const PROVIDER: &'static str = "gitlab";

#[derive(Debug, Deserialize)]
struct GitlabFileResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct GitlabRepoResponse {
    id: u32,
}

pub struct GitlabRepo {
    project_id: String,
    token: Option<GitlabToken>,
}

enum GitlabToken {
    Saved(String),
    FromEnv(String),
}

#[async_trait]
impl Repo for GitlabRepo {
    fn id() -> &'static str {
        PROVIDER
    }

    async fn get(&self, path: &str, repo_ref: &str) -> Result<String> {
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

impl From<GitlabRepo> for GenericRepo {
    fn from(gitlab_repo: GitlabRepo) -> Self {
        let (password, password_env) = match gitlab_repo.token {
            Some(GitlabToken::Saved(saved)) => (Some(saved), None),
            Some(GitlabToken::FromEnv(var)) => (None, Some(var)),
            _ => (None, None),
        };

        GenericRepo {
            provider: GitlabRepo::id().to_string(),
            uri: gitlab_repo.project_id,
            username: None,
            password,
            password_env,
        }
    }
}

impl TryFrom<GenericRepo> for GitlabRepo {
    type Error = anyhow::Error;
    fn try_from(repo: GenericRepo) -> Result<Self> {
        let token = match (repo.password, repo.password_env) {
            (Some(_), Some(_)) => bail!("Gitlab repo cannot have both passsword and password_env"),
            (Some(saved), None) => Some(GitlabToken::Saved(saved)),
            (None, Some(var)) => Some(GitlabToken::FromEnv(var)),
            _ => None,
        };

        Ok(Self {
            project_id: repo.uri,
            token,
        })
    }
}

pub async fn fetch_project(uri: &Url, token: Password) -> Result<GenericRepo> {
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
    };

    Ok(result.into())
}
