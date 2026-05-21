use async_trait::async_trait;
use nexara_core::{ToolCallRequest, ToolCallResult, ToolDescriptor, TrustProfile};
use nexara_registry::JsonFileInstalledSkillStore;
use nexara_runtime::{
    HostToolExecutor, NexaraRuntime, NexaraRuntimeConfig, StaticToolCatalog,
    StaticTrustPolicyResolver,
};
use nexara_server::{NexaraServerState, ServerAuth, ServerMetrics, memory_secret_store, router};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;

struct StatusExecutor;

#[async_trait]
impl HostToolExecutor<()> for StatusExecutor {
    async fn call_host_tool(
        &self,
        _tool: &ToolDescriptor,
        _request: ToolCallRequest,
        _context: (),
    ) -> Result<ToolCallResult, nexara_core::NexaraError> {
        Ok(ToolCallResult::new(json!({
            "status": "ok",
            "source": "local-http-server-example",
        })))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = Arc::new(NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![ToolDescriptor::read_only(
            "status",
            "Returns local example server status",
        )])),
        Arc::new(StatusExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    ));
    let tempdir = tempfile::tempdir()?;
    let state = NexaraServerState {
        runtime,
        auth: ServerAuth::require("dev-token"),
        installed_skills: JsonFileInstalledSkillStore::new(tempdir.path().join("skills.json")),
        secrets: memory_secret_store(),
        metrics: Arc::new(ServerMetrics::default()),
    };
    let app = router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3977));
    println!("list tools with:");
    println!("curl -H 'Authorization: Bearer dev-token' http://{addr}/v1/nexara/tools");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}
