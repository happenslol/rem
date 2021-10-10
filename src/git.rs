use crate::{repo::Repo, ScriptSource};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
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

    fn box_clone(&self) -> Box<dyn Repo> {
        Box::new(self.clone())
    }

    async fn fetch_script(&self, path: &str, rref: &str) -> Result<String> {
        Ok(cmd::fetch_script(&self.url, rref, path, false).await?)
    }
}

impl GitRepo {
    pub fn from_src(src: &ScriptSource) -> Box<dyn Repo> {
        Box::new(Self {
            url: src.repo.clone(),
        })
    }
}

mod cmd {
    use anyhow::{anyhow, bail, Context, Result};
    use async_process::{Command, Stdio};
    use sanitize_filename::{sanitize_with_options, Options as SanitizeOptions};
    use std::path::{Path, PathBuf};
    use tokio::fs;

    async fn get_ref_dir(repo: &str, rref: &str) -> Result<PathBuf> {
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
        let mut repo_path = sanitized_path.replace('@', ":");
        repo_path.push_str(&rref);

        cache_dir.push(repo_path);
        Ok(PathBuf::from(cache_dir))
    }

    async fn run_git_command(dir: &Path, args: &[&str]) -> Result<()> {
        let mut child = Command::new("git")
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()?;

        let status = child.status().await?;
        let output = child.output().await?;

        if status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("git command returned error: {}", stderr);
        }
    }

    pub async fn fetch_script(
        repo: &str,
        rref: &str,
        path: &str,
        force_fresh: bool,
    ) -> Result<String> {
        let mut ref_path = get_ref_dir(repo, rref).await?;
        if force_fresh && ref_path.is_dir() {
            fs::remove_dir_all(&ref_path).await?;
        }

        let is_clean = ref_path.is_dir()
            && run_git_command(&ref_path, &["diff", "--quiet"])
                .await
                .is_ok();

        if !is_clean {
            println!("cloning");
            fs::create_dir_all(&ref_path).await?;
            run_git_command(&ref_path, &["init"]).await?;
            run_git_command(&ref_path, &["remote", "add", "origin", repo]).await?;
            run_git_command(&ref_path, &["fetch", "--depth", "1", "origin", rref]).await?;
            run_git_command(&ref_path, &["checkout", "FETCH_HEAD"]).await?;
        }

        ref_path.push(path);
        Ok(fs::read_to_string(&ref_path).await?)
    }
}
