use crate::error::NexaraResult;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Allowed,
    DeniedNotFound,
    DeniedNotAllowed,
    DeniedReadOnly,
    DeniedTrustProfile,
    DeniedConfirmationRequired,
    DeniedPayloadTooLarge,
    DeniedConcurrencyLimit,
    DeniedInvalidParams,
    DeniedServiceUnavailable,
    DeniedPolicyContract,
    DeniedMissingCapabilities,
    DeniedNoMatchingAllow,
    DeniedPolicyConfirmationRequired,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Success,
    Error,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditRecord {
    pub timestamp_unix_ms: u64,
    pub caller: String,
    pub tool: String,
    pub trust_tier: String,
    pub action_class: String,
    pub trust_profile: String,
    pub policy_decision: PolicyDecision,
    pub outcome: AuditOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_hash: Option<String>,
    pub duration_ms: u64,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl AuditRecord {
    pub fn now(
        caller: impl Into<String>,
        tool: impl Into<String>,
        policy_decision: PolicyDecision,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            timestamp_unix_ms: now_unix_ms(),
            caller: caller.into(),
            tool: tool.into(),
            trust_tier: String::new(),
            action_class: String::new(),
            trust_profile: String::new(),
            policy_decision,
            outcome,
            result_hash: None,
            duration_ms: 0,
            metadata: BTreeMap::new(),
        }
    }
}

pub trait AuditSink: Send + Sync {
    fn record(&self, record: AuditRecord) -> NexaraResult<()>;
}

#[derive(Debug, Default)]
pub struct TracingAuditSink;

impl AuditSink for TracingAuditSink {
    fn record(&self, record: AuditRecord) -> NexaraResult<()> {
        tracing::info!(
            nexara.audit = true,
            caller = %record.caller,
            tool = %record.tool,
            policy_decision = ?record.policy_decision,
            outcome = ?record.outcome,
            result_hash = record.result_hash.as_deref().unwrap_or("-"),
            duration_ms = record.duration_ms,
            "nexara audit event"
        );
        Ok(())
    }
}

pub fn hash_result(value: &serde_json::Value) -> String {
    let serialized = serde_json::to_vec(value).unwrap_or_default();
    let digest = Sha256::digest(&serialized);
    digest
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
