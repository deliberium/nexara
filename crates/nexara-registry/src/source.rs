use crate::contract::SkillIndexSource;
use crate::manifest::{SignedSkillIndex, SignedSkillIndexEntry};
use async_trait::async_trait;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileSkillIndexSource {
    index_path: PathBuf,
}

impl FileSkillIndexSource {
    pub fn new(index_path: impl Into<PathBuf>) -> Self {
        Self {
            index_path: index_path.into(),
        }
    }
}

#[async_trait]
impl SkillIndexSource for FileSkillIndexSource {
    async fn fetch_index(&self) -> anyhow::Result<SignedSkillIndex> {
        let raw = std::fs::read_to_string(&self.index_path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    async fn fetch_manifest(&self, entry: &SignedSkillIndexEntry) -> anyhow::Result<String> {
        let path = PathBuf::from(&entry.manifest_url);
        Ok(std::fs::read_to_string(path)?)
    }
}

#[derive(Debug, Clone)]
pub struct HttpSkillIndexSource {
    client: reqwest::Client,
    index_url: String,
}

impl HttpSkillIndexSource {
    pub fn new(index_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            index_url: index_url.into(),
        }
    }
}

#[async_trait]
impl SkillIndexSource for HttpSkillIndexSource {
    async fn fetch_index(&self) -> anyhow::Result<SignedSkillIndex> {
        Ok(self
            .client
            .get(&self.index_url)
            .send()
            .await?
            .json()
            .await?)
    }

    async fn fetch_manifest(&self, entry: &SignedSkillIndexEntry) -> anyhow::Result<String> {
        Ok(self
            .client
            .get(&entry.manifest_url)
            .send()
            .await?
            .text()
            .await?)
    }
}
