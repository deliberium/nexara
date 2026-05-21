use async_trait::async_trait;
use nexara_core::{ToolCallRequest, ToolCallResult, ToolDescriptor, TrustProfile};
use nexara_runtime::{
    HostToolExecutor, NexaraRuntime, NexaraRuntimeConfig, StaticToolCatalog,
    StaticTrustPolicyResolver,
};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone)]
struct RequestContext {
    actor: String,
}

struct EchoExecutor;

#[async_trait]
impl HostToolExecutor<RequestContext> for EchoExecutor {
    async fn call_host_tool(
        &self,
        _tool: &ToolDescriptor,
        request: ToolCallRequest,
        context: RequestContext,
    ) -> Result<ToolCallResult, nexara_core::NexaraError> {
        Ok(ToolCallResult::new(json!({
            "actor": context.actor,
            "params": request.params,
        })))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![ToolDescriptor::read_only(
            "echo",
            "Echoes request parameters for an embedded host",
        )])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    );

    let visible = runtime.list_tools(Some("please echo this"), &["read".to_string()]);
    println!("{}", serde_json::to_string_pretty(&visible)?);

    let result = runtime
        .call_tool(
            ToolCallRequest {
                tool: "echo".to_string(),
                params: json!({ "message": "hello from embedded Nexara" }),
            },
            RequestContext {
                actor: "example-user".to_string(),
            },
        )
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
