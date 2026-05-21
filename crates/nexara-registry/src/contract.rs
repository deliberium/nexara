use crate::manifest::{SignedSkillIndex, SignedSkillIndexEntry};
use async_trait::async_trait;

#[async_trait]
pub trait SkillIndexSource: Send + Sync {
    async fn fetch_index(&self) -> anyhow::Result<SignedSkillIndex>;
    async fn fetch_manifest(&self, entry: &SignedSkillIndexEntry) -> anyhow::Result<String>;
}

pub trait SignatureVerifier: Send + Sync {
    fn verify_index(&self, index: &SignedSkillIndex) -> anyhow::Result<()>;
    fn verify_manifest_bytes(&self, bytes: &[u8], expected_sha256: &str) -> anyhow::Result<()>;
}
