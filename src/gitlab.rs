#![allow(dead_code)]

use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::{
    convert::{From, TryFrom},
    env,
};
use url::Url;

use crate::repo::{GenericRepo, Repo};

#[derive(Debug, Deserialize)]
struct GitlabFileResponse {
    content: String,
}

struct GitlabRepo {
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
        "gitlab"
    }

    async fn get(&self, path: &str) -> Result<String> {
        let script_url = format!(
            "https://gitlab.com/api/v4/projects/{}/repository/files/{}?ref=main",
            self.project_id, path,
        );

        let req = reqwest::Client::new().get(script_url);
        let req = match &self.token {
            Some(GitlabToken::Saved(saved)) => req.header("PRIVATE-TOKEN", saved),
            Some(GitlabToken::FromEnv(var)) => req.header("PRIVATE-TOKEN", env::var(var)?),
            _ => req,
        };

        let resp = req.send().await?.json::<GitlabFileResponse>().await?;
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

#[derive(Debug, Deserialize)]
struct GitlabRepoResponse {
    id: u32,
}

pub async fn fetch_project(uri: &Url, token: Option<String>) -> Result<GenericRepo> {
    let without_leading_slash = uri.path().trim_start_matches('/');
    let encoded_uri = urlencoding::encode(without_leading_slash);
    let repo_url = format!("https://gitlab.com/api/v4/projects/{}", encoded_uri);
    let req = reqwest::Client::new().get(repo_url);

    let req = match token {
        Some(token) => req.header("PRIVATE-TOKEN", token),
        _ => req,
    };

    let resp = req.send().await?.json::<GitlabRepoResponse>().await?;
    let result = GitlabRepo {
        project_id: format!("{}", resp.id),
        token: None, // TODO
    };

    Ok(result.into())
}
