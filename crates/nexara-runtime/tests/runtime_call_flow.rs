use async_trait::async_trait;
use nexara_core::{
    ActionClass, AuditRecord, AuditSink, CapabilityPattern, CapabilitySensitivity, NexaraError,
    PolicyContract, PolicyDecision, PolicyDefaults, PolicyEffect, PolicyRule, PolicySelector,
    PolicySource, PolicySourceKind, ToolCallRequest, ToolCallResult, ToolDescriptor, TrustProfile,
    TrustTier,
};
use nexara_runtime::{
    HostToolExecutor, NexaraRuntime, NexaraRuntimeConfig, RuntimeLifecycleHooks, StaticToolCatalog,
    StaticTrustPolicyResolver,
};
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

#[derive(Clone)]
struct Ctx;

struct EchoExecutor;

#[async_trait]
impl HostToolExecutor<Ctx> for EchoExecutor {
    async fn call_host_tool(
        &self,
        _tool: &ToolDescriptor,
        request: ToolCallRequest,
        _context: Ctx,
    ) -> Result<ToolCallResult, nexara_core::NexaraError> {
        Ok(ToolCallResult::new(json!({ "echo": request.params })))
    }
}

struct FailingExecutor;

#[async_trait]
impl HostToolExecutor<Ctx> for FailingExecutor {
    async fn call_host_tool(
        &self,
        _tool: &ToolDescriptor,
        _request: ToolCallRequest,
        _context: Ctx,
    ) -> Result<ToolCallResult, nexara_core::NexaraError> {
        Err(NexaraError::ExecutionFailed("boom".to_string()))
    }
}

struct BlockingExecutor {
    started: Arc<Notify>,
    release: Arc<Notify>,
}

#[async_trait]
impl HostToolExecutor<Ctx> for BlockingExecutor {
    async fn call_host_tool(
        &self,
        _tool: &ToolDescriptor,
        _request: ToolCallRequest,
        _context: Ctx,
    ) -> Result<ToolCallResult, nexara_core::NexaraError> {
        self.started.notify_one();
        self.release.notified().await;
        Ok(ToolCallResult::new(json!({ "done": true })))
    }
}

#[derive(Default)]
struct RecordingAuditSink {
    records: Mutex<Vec<AuditRecord>>,
}

impl AuditSink for RecordingAuditSink {
    fn record(&self, record: AuditRecord) -> nexara_core::NexaraResult<()> {
        self.records.lock().unwrap().push(record);
        Ok(())
    }
}

#[derive(Default)]
struct RecordingHooks {
    before: Mutex<Vec<String>>,
    after: Mutex<Vec<String>>,
}

#[async_trait]
impl RuntimeLifecycleHooks<Ctx> for RecordingHooks {
    async fn before_call(&self, tool: &ToolDescriptor, _request: &ToolCallRequest, _context: &Ctx) {
        self.before.lock().unwrap().push(tool.name.clone());
    }

    async fn after_call(
        &self,
        tool: &ToolDescriptor,
        _request: &ToolCallRequest,
        outcome: Result<&ToolCallResult, &NexaraError>,
        _context: &Ctx,
    ) {
        let suffix = if outcome.is_ok() { "ok" } else { "err" };
        self.after
            .lock()
            .unwrap()
            .push(format!("{}:{suffix}", tool.name));
    }
}

fn descriptor(action_class: ActionClass) -> ToolDescriptor {
    ToolDescriptor {
        name: "echo".to_string(),
        description: "Echo params".to_string(),
        tags: vec!["echo".to_string()],
        scopes: vec!["read".to_string()],
        trust_tier: TrustTier::Builtin,
        action_class,
        enabled: true,
        input_schema: None,
        guidance: None,
        capabilities: Vec::new(),
        effects: Vec::new(),
    }
}

fn semantic_descriptor(
    name: &str,
    capability_id: &str,
    resource_path: &[&str],
    operation: &str,
    action_class: ActionClass,
) -> ToolDescriptor {
    let mut descriptor = descriptor(action_class);
    descriptor.name = name.to_string();
    descriptor.capabilities = vec![nexara_core::ToolCapability {
        id: capability_id.to_string(),
        display_name: Some(format!("{operation} {}", resource_path.join(" "))),
        description: Some(format!("Capability {capability_id}")),
        resource: resource_path[0].to_string(),
        resource_path: resource_path
            .iter()
            .map(|part| (*part).to_string())
            .collect(),
        operation: operation.to_string(),
        action_class,
        sensitivity: CapabilitySensitivity::Business,
        aliases: Vec::new(),
    }];
    descriptor
}

fn orders_policy() -> PolicyContract {
    PolicyContract {
        id: "orders-never-refunds".to_string(),
        version: "0.1.0".to_string(),
        source: PolicySource {
            kind: PolicySourceKind::OneLine,
            text: "reads orders, never refunds".to_string(),
        },
        rules: vec![
            PolicyRule {
                id: "allow-orders-read".to_string(),
                effect: PolicyEffect::Allow,
                selector: PolicySelector {
                    capabilities: vec![CapabilityPattern("orders.read".to_string())],
                    ..PolicySelector::default()
                },
                condition: None,
                reason: "reads orders".to_string(),
            },
            PolicyRule {
                id: "deny-refunds".to_string(),
                effect: PolicyEffect::Deny,
                selector: PolicySelector {
                    capabilities: vec![CapabilityPattern("orders.refunds.create".to_string())],
                    ..PolicySelector::default()
                },
                condition: None,
                reason: "never refunds".to_string(),
            },
        ],
        defaults: PolicyDefaults::default(),
        diagnostics: Vec::new(),
        created_at: None,
    }
}

#[tokio::test]
async fn read_call_succeeds() {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Read)])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    );
    let result = runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({ "hello": "world" }),
            },
            Ctx,
        )
        .await
        .unwrap();
    assert_eq!(result.result["echo"]["hello"], "world");
}

#[tokio::test]
async fn policy_contract_allows_matching_read_capability() {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![semantic_descriptor(
            "shopify.get_order",
            "orders.read",
            &["orders"],
            "read",
            ActionClass::Read,
        )])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    )
    .with_policy_contract(orders_policy());

    let result = runtime
        .call_tool(
            ToolCallRequest {
                tool: "shopify.get_order".to_string(),
                params: json!({ "order_id": "1001" }),
            },
            Ctx,
        )
        .await
        .unwrap();

    assert_eq!(result.result["echo"]["order_id"], "1001");
}

#[tokio::test]
async fn policy_contract_denies_refund_capability() {
    let audit = Arc::new(RecordingAuditSink::default());
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![semantic_descriptor(
            "shopify.create_refund",
            "orders.refunds.create",
            &["orders", "refunds"],
            "create",
            ActionClass::Write,
        )])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::FullOperator)),
    )
    .with_audit_sink(audit.clone())
    .with_policy_contract(orders_policy());

    let err = runtime
        .call_tool(
            ToolCallRequest {
                tool: "shopify.create_refund".to_string(),
                params: json!({ "_confirmed": true }),
            },
            Ctx,
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, NexaraError::TrustPolicyDenied(message) if message.contains("never refunds"))
    );

    let records = audit.records.lock().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].tool, "shopify.create_refund");
    assert_eq!(
        records[0].policy_decision,
        PolicyDecision::DeniedPolicyContract
    );
    assert_eq!(records[0].outcome, nexara_core::AuditOutcome::Rejected);
    assert_eq!(
        records[0].metadata.get("policy.matched_rules"),
        Some(&"deny-refunds".to_string())
    );
    assert!(
        records[0]
            .metadata
            .get("policy.explanation")
            .is_some_and(|message| message.contains("never refunds"))
    );
}

#[tokio::test]
async fn policy_contract_denies_missing_semantic_capabilities() {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Read)])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    )
    .with_policy_contract(orders_policy());

    let err = runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({}),
            },
            Ctx,
        )
        .await
        .unwrap_err();

    assert!(
        matches!(err, NexaraError::TrustPolicyDenied(message) if message.contains("semantic capability"))
    );
}

#[tokio::test]
async fn verified_policy_contract_runs_host_verifier_before_activation() {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![semantic_descriptor(
            "shopify.get_order",
            "orders.read",
            &["orders"],
            "read",
            ActionClass::Read,
        )])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    )
    .with_verified_policy_contract(orders_policy(), |contract| {
        if contract.id == "orders-never-refunds" {
            Ok(())
        } else {
            Err(NexaraError::TrustPolicyDenied("bad policy".to_string()))
        }
    })
    .unwrap();

    runtime
        .call_tool(
            ToolCallRequest {
                tool: "shopify.get_order".to_string(),
                params: json!({}),
            },
            Ctx,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn write_requires_confirmation() {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Write)])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(
            TrustProfile::ActWithConfirmation,
        )),
    );
    let err = runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({}),
            },
            Ctx,
        )
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        nexara_core::NexaraError::ConfirmationRequired(_)
    ));
}

#[tokio::test]
async fn confirmed_write_strips_confirmation_marker_before_execution() {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Write)])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(
            TrustProfile::ActWithConfirmation,
        )),
    );

    let result = runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({ "_confirmed": true, "payload": "kept" }),
            },
            Ctx,
        )
        .await
        .unwrap();

    assert_eq!(result.result["echo"], json!({ "payload": "kept" }));
}

#[tokio::test]
async fn payload_limit_rejects_oversized_request_before_execution() {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig {
            max_payload_bytes: 4,
            ..NexaraRuntimeConfig::default()
        },
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Read)])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    );

    let err = runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({ "too": "large" }),
            },
            Ctx,
        )
        .await
        .unwrap_err();

    assert!(matches!(err, NexaraError::PayloadTooLarge(_)));
}

#[tokio::test]
async fn audit_and_lifecycle_hooks_record_successful_and_failed_calls() {
    let audit = Arc::new(RecordingAuditSink::default());
    let hooks = Arc::new(RecordingHooks::default());
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Read)])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    )
    .with_audit_sink(audit.clone())
    .with_lifecycle_hooks(hooks.clone());

    runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({ "hello": "world" }),
            },
            Ctx,
        )
        .await
        .unwrap();

    {
        let records = audit.records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tool, "echo");
        assert_eq!(records[0].policy_decision, PolicyDecision::Allowed);
        assert!(records[0].result_hash.is_some());
    }

    assert_eq!(hooks.before.lock().unwrap().as_slice(), ["echo"]);
    assert_eq!(hooks.after.lock().unwrap().as_slice(), ["echo:ok"]);

    let failing_audit = Arc::new(RecordingAuditSink::default());
    let failing_hooks = Arc::new(RecordingHooks::default());
    let failing_runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Read)])),
        Arc::new(FailingExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    )
    .with_audit_sink(failing_audit.clone())
    .with_lifecycle_hooks(failing_hooks.clone());

    let err = failing_runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({}),
            },
            Ctx,
        )
        .await
        .unwrap_err();
    assert!(matches!(err, NexaraError::ExecutionFailed(_)));
    assert_eq!(failing_audit.records.lock().unwrap().len(), 1);
    assert_eq!(failing_hooks.after.lock().unwrap().as_slice(), ["echo:err"]);
}

#[tokio::test]
async fn concurrency_limit_rejects_second_call_while_first_is_running() {
    let started = Arc::new(Notify::new());
    let release = Arc::new(Notify::new());
    let runtime = Arc::new(NexaraRuntime::new(
        NexaraRuntimeConfig {
            max_concurrent_calls: 1,
            ..NexaraRuntimeConfig::default()
        },
        Arc::new(StaticToolCatalog::new(vec![descriptor(ActionClass::Read)])),
        Arc::new(BlockingExecutor {
            started: started.clone(),
            release: release.clone(),
        }),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    ));

    let first_runtime = runtime.clone();
    let first = tokio::spawn(async move {
        first_runtime
            .call_tool(
                ToolCallRequest {
                    tool: "echo".to_string(),
                    params: json!({}),
                },
                Ctx,
            )
            .await
    });
    started.notified().await;

    let err = runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({}),
            },
            Ctx,
        )
        .await
        .unwrap_err();

    assert_eq!(err, NexaraError::ConcurrencyLimitExceeded);
    release.notify_one();
    first.await.unwrap().unwrap();
}
