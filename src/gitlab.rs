use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::{
    convert::{From, TryFrom},
    env,
};

use crate::repo::{GenericRepo, Repo};

#[derive(Debug, Deserialize)]
struct GitlabFileResponse {
    content: String,
}

struct GitlabRepo {
    project_id: String,
    token: GitlabToken,
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
        let gitlab_token = match &self.token {
            GitlabToken::Saved(saved) => saved.to_string(),
            GitlabToken::FromEnv(var) => env::var(var)?,
        };

        let script_url = format!(
            "https://gitlab.com/api/v4/projects/{}/repository/files/{}?ref=main",
            self.project_id, path,
        );

        let resp = reqwest::Client::new()
            .get(script_url)
            .header("PRIVATE-TOKEN", gitlab_token)
            .send()
            .await?
            .json::<GitlabFileResponse>()
            .await?;

        let decoded_content = base64::decode(resp.content)?;
        Ok(String::from_utf8(decoded_content)?)
    }
}

impl From<GitlabRepo> for GenericRepo {
    fn from(gitlab_repo: GitlabRepo) -> Self {
        let (password, password_env) = match gitlab_repo.token {
            GitlabToken::Saved(saved) => (Some(saved), None),
            GitlabToken::FromEnv(var) => (None, Some(var)),
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
            (None, None) => bail!("Gitlab repo requires passsword or password_env"),
            (Some(_), Some(_)) => bail!("Gitlab repo cannot have both passsword and password_env"),
            (Some(saved), None) => GitlabToken::Saved(saved),
            (None, Some(var)) => GitlabToken::FromEnv(var),
        };

        Ok(Self {
            project_id: repo.uri,
            token,
        })
    }
}
