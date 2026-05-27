use nexara_core::{ActionClass, CapabilitySensitivity, ToolEffect};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillRuntime {
    BuiltinBundle,
    RemoteMcp,
    NexaraHttp,
    WasmComponent,
    NativeDynamicLibrary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillPublisherTrust {
    Official,
    Verified,
    Community,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkillConfirmationPolicy {
    Never,
    Always,
    OnWrite,
    OnFirstUse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HostRequirements {
    #[serde(flatten)]
    pub requirements: BTreeMap<String, String>,
}

impl HostRequirements {
    pub fn requirement(&self, host: &str) -> Option<&str> {
        self.requirements.get(host).map(String::as_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builtin_bundle_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_auth_secret_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillCapabilityDescriptor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(default)]
    pub resource_path: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_class: Option<ActionClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<CapabilitySensitivity>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub intent_examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SkillPresentation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_summary: Option<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub description: String,
    pub runtime: SkillRuntime,
    pub trust_tier: SkillPublisherTrust,
    pub host_requirements: HostRequirements,
    pub mcp_version: String,
    pub action_class: ActionClass,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub confirmation: SkillConfirmationPolicy,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    #[serde(default)]
    pub requires_secrets: Vec<String>,
    pub source: SkillSource,
    #[serde(default)]
    pub presentation: SkillPresentation,
    #[serde(default)]
    pub capabilities: Vec<SkillCapabilityDescriptor>,
    #[serde(default)]
    pub effects: Vec<ToolEffect>,
}

pub type SkillManifest = SkillMetadata;

#[derive(Debug, Deserialize)]
struct RawSkillManifest {
    name: String,
    version: String,
    publisher: String,
    description: String,
    runtime: SkillRuntime,
    trust_tier: SkillPublisherTrust,
    host_requirements: HostRequirements,
    mcp_version: String,
    source: SkillSource,
    policy: RawSkillPolicy,
    #[serde(default)]
    presentation: SkillPresentation,
    #[serde(default)]
    capabilities: Vec<SkillCapabilityDescriptor>,
    #[serde(default)]
    effects: Vec<ToolEffect>,
}

#[derive(Debug, Deserialize)]
struct RawSkillPolicy {
    action_class: ActionClass,
    scopes: Vec<String>,
    confirmation: SkillConfirmationPolicy,
    #[serde(default)]
    allowed_hosts: Vec<String>,
    #[serde(default)]
    requires_secrets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedSkillIndex {
    pub schema_version: String,
    pub generated_at: String,
    pub publisher: SignedSkillIndexPublisher,
    pub signature: SignedSkillIndexSignature,
    pub skills: Vec<SignedSkillIndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedSkillIndexPublisher {
    pub id: String,
    pub display_name: String,
    pub public_key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedSkillIndexSignature {
    pub algorithm: String,
    pub key_id: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedSkillIndexEntry {
    pub id: String,
    pub version: String,
    pub manifest_url: String,
    pub manifest_sha256: String,
    pub trust_tier: SkillPublisherTrust,
    pub runtime: SkillRuntime,
    pub action_class: ActionClass,
    pub scopes: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub requires_secrets: Vec<String>,
    pub requires_confirmation: bool,
    pub host_requirements: HostRequirements,
    pub mcp_version: String,
    pub summary: String,
}

pub fn parse_skill_manifest_str(input: &str) -> anyhow::Result<SkillMetadata> {
    if input.contains("supported_host") {
        anyhow::bail!("supported_host is not supported; use host_requirements");
    }
    let raw: RawSkillManifest = toml::from_str(input)?;
    let id = format!("{}/{}", raw.publisher, raw.name);
    Ok(SkillMetadata {
        id,
        name: raw.name,
        version: raw.version,
        publisher: raw.publisher,
        description: raw.description,
        runtime: raw.runtime,
        trust_tier: raw.trust_tier,
        host_requirements: raw.host_requirements,
        mcp_version: raw.mcp_version,
        action_class: raw.policy.action_class,
        scopes: raw.policy.scopes,
        confirmation: raw.policy.confirmation,
        allowed_hosts: raw.policy.allowed_hosts,
        requires_secrets: raw.policy.requires_secrets,
        source: raw.source,
        presentation: raw.presentation,
        capabilities: raw.capabilities,
        effects: raw.effects,
    })
}

pub fn validate_host_requirements(
    requirements: &HostRequirements,
    host: &str,
    version: &str,
) -> anyhow::Result<()> {
    let Some(req) = requirements.requirement(host) else {
        return Ok(());
    };
    let req = VersionReq::parse(req)?;
    let version = Version::parse(version)?;
    if req.matches(&version) {
        Ok(())
    } else {
        anyhow::bail!("host '{host}' version {version} does not satisfy {req}")
    }
}
