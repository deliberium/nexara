use crate::policy::{ActionClass, TrustTier};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolGuidance {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequencing_hints: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_recovery: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDescriptor {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub trust_tier: TrustTier,
    pub action_class: ActionClass,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidance: Option<ToolGuidance>,
    #[serde(default)]
    pub capabilities: Vec<ToolCapability>,
    #[serde(default)]
    pub effects: Vec<ToolEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCapability {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub resource: String,
    #[serde(default)]
    pub resource_path: Vec<String>,
    pub operation: String,
    pub action_class: ActionClass,
    pub sensitivity: CapabilitySensitivity,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilitySensitivity {
    Public,
    Internal,
    Business,
    Personal,
    Financial,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolEffect {
    pub resource: String,
    pub operation: String,
    pub writes: bool,
    pub external_side_effect: bool,
    pub financial_side_effect: bool,
    pub irreversible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ToolRef {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolSelectionRequest {
    pub prompt: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub max_tools: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallRequest {
    pub tool: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCallResult {
    #[serde(default)]
    pub result: Value,
}

impl ToolCallResult {
    pub fn new(result: Value) -> Self {
        Self { result }
    }
}

impl ToolDescriptor {
    pub fn read_only(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            tags: Vec::new(),
            scopes: vec!["read".to_string()],
            trust_tier: TrustTier::Builtin,
            action_class: ActionClass::Read,
            enabled: true,
            input_schema: None,
            guidance: None,
            capabilities: Vec::new(),
            effects: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: ToolCapability) -> Self {
        self.capabilities.push(capability);
        self
    }
}

impl ToolCapability {
    pub fn new(
        id: impl Into<String>,
        resource: impl Into<String>,
        operation: impl Into<String>,
        action_class: ActionClass,
    ) -> Self {
        let id = id.into();
        let resource = resource.into();
        let operation = operation.into();
        Self {
            id,
            display_name: None,
            description: None,
            resource_path: vec![resource.clone()],
            resource,
            operation,
            action_class,
            sensitivity: CapabilitySensitivity::Internal,
            aliases: Vec::new(),
        }
    }
}
