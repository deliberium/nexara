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
        }
    }
}
