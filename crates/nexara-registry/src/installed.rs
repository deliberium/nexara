use crate::manifest::SkillMetadata;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct InstalledSkillState {
    pub installed: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstalledSkillRecord {
    pub metadata: SkillMetadata,
    pub state: InstalledSkillState,
}

pub trait InstalledSkillStore: Send + Sync {
    fn list(&self) -> anyhow::Result<Vec<InstalledSkillRecord>>;
    fn get(&self, skill_id: &str) -> anyhow::Result<Option<InstalledSkillRecord>>;
    fn put(&self, record: InstalledSkillRecord) -> anyhow::Result<()>;
    fn remove(&self, skill_id: &str) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct JsonFileInstalledSkillStore {
    path: PathBuf,
}

impl JsonFileInstalledSkillStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn ensure_parent(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

impl InstalledSkillStore for JsonFileInstalledSkillStore {
    fn list(&self) -> anyhow::Result<Vec<InstalledSkillRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read(&self.path)?;
        Ok(serde_json::from_slice(&raw)?)
    }

    fn get(&self, skill_id: &str) -> anyhow::Result<Option<InstalledSkillRecord>> {
        Ok(self
            .list()?
            .into_iter()
            .find(|record| record.metadata.id == skill_id))
    }

    fn put(&self, record: InstalledSkillRecord) -> anyhow::Result<()> {
        self.ensure_parent()?;
        let mut records = self.list()?;
        records.retain(|existing| existing.metadata.id != record.metadata.id);
        records.push(record);
        records.sort_by(|left, right| left.metadata.id.cmp(&right.metadata.id));
        fs::write(&self.path, serde_json::to_vec_pretty(&records)?)?;
        Ok(())
    }

    fn remove(&self, skill_id: &str) -> anyhow::Result<()> {
        self.ensure_parent()?;
        let mut records = self.list()?;
        records.retain(|existing| existing.metadata.id != skill_id);
        fs::write(&self.path, serde_json::to_vec_pretty(&records)?)?;
        Ok(())
    }
}

pub fn ensure_parent_dir(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
