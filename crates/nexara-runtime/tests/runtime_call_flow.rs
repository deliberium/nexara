use async_trait::async_trait;
use nexara_core::{
    ActionClass, ToolCallRequest, ToolCallResult, ToolDescriptor, TrustProfile, TrustTier,
};
use nexara_runtime::{
    HostToolExecutor, NexaraRuntime, NexaraRuntimeConfig, StaticToolCatalog,
    StaticTrustPolicyResolver,
};
use serde_json::json;
use std::sync::Arc;

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
