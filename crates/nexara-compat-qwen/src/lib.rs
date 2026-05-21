use nexara_remote::RemoteEndpoint;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QwenCompatibilityProfile {
    pub is_qwen_family: bool,
    pub tool_prompt_guidance: Option<String>,
}

pub fn build_qwen_compatibility_profile(model_name: &str) -> QwenCompatibilityProfile {
    let normalized = model_name.to_ascii_lowercase();
    let is_qwen_family = normalized.contains("qwen");
    QwenCompatibilityProfile {
        is_qwen_family,
        tool_prompt_guidance: is_qwen_family.then(|| {
            "When a Nexara tool is useful, request it with compact JSON arguments only.".to_string()
        }),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QwenMcpConfig {
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: BTreeMap<String, QwenMcpServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QwenMcpServer {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QwenImportRecord {
    pub name: String,
    pub endpoint: RemoteEndpoint,
    pub approved: bool,
}

pub trait QwenApprovalStore: Send + Sync {
    fn is_approved(&self, name: &str, endpoint: &RemoteEndpoint) -> anyhow::Result<bool>;
    fn set_approved(
        &self,
        name: &str,
        endpoint: &RemoteEndpoint,
        approved: bool,
    ) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Default)]
pub struct MemoryQwenApprovalStore {
    approvals: std::sync::Arc<std::sync::Mutex<BTreeMap<String, bool>>>,
}

impl MemoryQwenApprovalStore {
    fn key(name: &str, endpoint: &RemoteEndpoint) -> String {
        format!("{name}\u{1f}{base}", base = endpoint.base_url)
    }
}

impl QwenApprovalStore for MemoryQwenApprovalStore {
    fn is_approved(&self, name: &str, endpoint: &RemoteEndpoint) -> anyhow::Result<bool> {
        Ok(self
            .approvals
            .lock()
            .unwrap()
            .get(&Self::key(name, endpoint))
            .copied()
            .unwrap_or(false))
    }

    fn set_approved(
        &self,
        name: &str,
        endpoint: &RemoteEndpoint,
        approved: bool,
    ) -> anyhow::Result<()> {
        self.approvals
            .lock()
            .unwrap()
            .insert(Self::key(name, endpoint), approved);
        Ok(())
    }
}

pub fn import_qwen_mcp_servers(config: &str) -> anyhow::Result<Vec<QwenImportRecord>> {
    import_qwen_mcp_servers_with_approvals(config, None)
}

pub fn import_qwen_mcp_servers_with_approvals(
    config: &str,
    approvals: Option<&dyn QwenApprovalStore>,
) -> anyhow::Result<Vec<QwenImportRecord>> {
    let parsed: QwenMcpConfig = serde_json::from_str(config)?;
    let mut records = Vec::new();
    for (name, server) in parsed.mcp_servers {
        if server.command.is_some() {
            anyhow::bail!("command/process MCP transport is not supported in Nexara v0.1");
        }
        let Some(url) = server.url else {
            continue;
        };
        let endpoint = RemoteEndpoint { base_url: url };
        let approved = approvals
            .map(|store| store.is_approved(&name, &endpoint))
            .transpose()?
            .unwrap_or(false);
        records.push(QwenImportRecord {
            name,
            endpoint,
            approved,
        });
    }
    records.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(records)
}
