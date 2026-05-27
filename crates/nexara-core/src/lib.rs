pub mod audit;
pub mod broker;
pub mod error;
pub mod policy;
pub mod policy_contract;
pub mod registry;
pub mod tool;
pub mod validation;

pub const NEXARA_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use audit::{
    AuditOutcome, AuditRecord, AuditSink, PolicyDecision, TracingAuditSink, hash_result,
};
pub use broker::{BrokerScoredTool, ToolBroker, ToolSelectionExplanation, ToolUsageSignalProvider};
pub use error::{NexaraError, NexaraResult};
pub use policy::{ActionClass, ConfirmationPolicy, EffectiveTrustPolicy, TrustProfile, TrustTier};
pub use policy_contract::{
    ActionPattern, CapabilityPattern, MatchedPolicyRule, PolicyCondition, PolicyContract,
    PolicyDefaults, PolicyDiagnostic, PolicyDiagnosticCode, PolicyDiagnosticSeverity, PolicyEffect,
    PolicyEvaluation, PolicyEvaluationDecision, PolicyRule, PolicySelector, PolicySource,
    PolicySourceKind, ResourcePattern, ScopePattern, ToolPattern, matched_rules_metadata,
    pattern_matches,
};
pub use registry::ToolRegistry;
pub use tool::{
    CapabilitySensitivity, ToolCallRequest, ToolCallResult, ToolCapability, ToolDescriptor,
    ToolEffect, ToolGuidance, ToolRef, ToolSelectionRequest,
};
pub use validation::{
    validate_capability_id, validate_scope_name, validate_tool_capability,
    validate_tool_descriptor, validate_tool_name,
};
