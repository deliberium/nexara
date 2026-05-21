pub mod audit;
pub mod broker;
pub mod error;
pub mod policy;
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
pub use registry::ToolRegistry;
pub use tool::{
    ToolCallRequest, ToolCallResult, ToolDescriptor, ToolGuidance, ToolRef, ToolSelectionRequest,
};
pub use validation::{validate_scope_name, validate_tool_descriptor, validate_tool_name};
