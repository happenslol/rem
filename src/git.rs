use crate::repo::Repo;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GitRepo {
    url: String,
}

pub const PROVIDER: &'static str = "git";

#[async_trait]
#[typetag::serde]
impl Repo for GitRepo {
    fn provider(&self) -> &'static str {
        PROVIDER
    }

    fn readable(&self) -> String {
        format!("{}", self.url)
    }

    async fn fetch_script(&self, path: &str, repo_ref: &str) -> anyhow::Result<String> {
        todo!()
    }
}

impl GitRepo {
}
