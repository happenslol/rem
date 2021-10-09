use crate::repo::Repo;
use anyhow::{anyhow, Result};
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

impl GitRepo {}

pub async fn fetch_project() -> Result<Box<dyn Repo>> {
    todo!()
}

mod cmd {
    use anyhow::{anyhow, Context, Result};
    use async_process::{Command, ExitStatus, Stdio};
    use sanitize_filename::{sanitize_with_options, Options as SanitizeOptions};
    use std::path::{Path, PathBuf};
    use tokio::{fs, io::BufReader};

    async fn get_ref_dir(repo: String, rref: String) -> Result<PathBuf> {
        let mut cache_dir = dirs::cache_dir().ok_or(anyhow!("Failed to get cache dir"))?;
        cache_dir.push("rem");
        if !cache_dir.is_dir() {
            fs::create_dir(&cache_dir)
                .await
                .context("Failed to create cache dir")?;
        }

        let mut sanitized_path = sanitize_with_options(
            repo,
            SanitizeOptions {
                truncate: true,
                windows: false,
                replacement: ":",
            },
        );

        sanitized_path.push_str("@");
        sanitized_path.push_str(&rref);

        Ok(PathBuf::from(sanitized_path))
    }

    async fn run_git_command(dir: &Path, args: &[&str]) -> Result<()> {
        let child = Command::new("git")
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()?;

        Ok(())
    }

    async fn is_clean(ref_path: &Path) -> Result<bool> {
        let mut cmd = Command::new("git");
        cmd.args(&["diff", "--quiet"]);
        cmd.current_dir(ref_path);
        cmd.output().await?;

        let result = cmd.status().await?;
        Ok(result.success())
    }

    pub async fn fetch_script(
        repo: String,
        rref: String,
        path: String,
        force_fresh: bool,
    ) -> Result<()> {
        let ref_path = get_ref_dir(repo, rref).await?;
        if force_fresh && ref_path.is_dir() {
            fs::remove_dir_all(&ref_path).await?;
        }

        if !ref_path.is_dir() || !is_clean(&ref_path).await? {
        }

        Ok(())
    }
}
