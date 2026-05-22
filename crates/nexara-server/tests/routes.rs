use std::sync::Arc;

use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use nexara_core::{ToolCallRequest, ToolCallResult, ToolDescriptor, TrustProfile};
use nexara_runtime::{
    HostToolExecutor, NexaraRuntime, NexaraRuntimeConfig, StaticToolCatalog,
    StaticTrustPolicyResolver,
};
use nexara_server::{
    NexaraServerState, ServerAuth, ServerMetrics, default_installed_store, memory_secret_store,
    router,
};
use serde_json::json;
use tower::ServiceExt;

const MANIFEST: &str = r#"
name = "fixture-skill"
version = "0.1.0"
publisher = "example"
description = "Fixture skill"
runtime = "nexara_http"
trust_tier = "community"
mcp_version = "2025-12"

[host_requirements]
nexara = ">=0.1.0"

[source]
server_url = "https://tools.example.com"

[policy]
action_class = "read"
scopes = ["read"]
confirmation = "never"
allowed_hosts = ["tools.example.com"]
requires_secrets = ["API_TOKEN"]
"#;

#[tokio::test]
async fn protected_routes_reject_missing_auth_and_report_metrics() {
    let app = test_router();

    let response = app
        .clone()
        .oneshot(request("GET", "/v1/nexara/tools", None, None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let metrics = json_response(
        app.oneshot(request("GET", "/metrics", None, None))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(metrics["unauthorized_requests"], 1);
}

#[tokio::test]
async fn list_and_call_tool_routes_use_runtime() {
    let app = test_router();

    let tools = json_response(
        app.clone()
            .oneshot(request(
                "GET",
                "/v1/nexara/tools?prompt=echo",
                Some("secret"),
                None,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(tools["status"], "ok");
    assert_eq!(tools["tools"][0]["name"], "echo");

    let result = json_response(
        app.clone()
            .oneshot(request(
                "POST",
                "/v1/nexara/tools/call",
                Some("secret"),
                Some(json!({ "tool": "echo", "params": { "hello": "world" } })),
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(result["status"], "ok");
    assert_eq!(result["result"]["echo"]["hello"], "world");

    let metrics = json_response(
        app.oneshot(request("GET", "/metrics", None, None))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(metrics["tool_list_requests"], 1);
    assert_eq!(metrics["tool_call_requests"], 1);
}

#[tokio::test]
async fn admin_install_enable_disable_readiness_and_secret_flow() {
    let app = test_router();
    let skill_id = "example%2Ffixture-skill";

    let install = json_response(
        app.clone()
            .oneshot(request(
                "POST",
                "/admin/nexara/skills/install",
                Some("secret"),
                Some(json!({ "manifest_toml": MANIFEST, "enabled": false })),
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(install["status"], "ok");

    let skills = json_response(
        app.clone()
            .oneshot(request("GET", "/admin/nexara/skills", Some("secret"), None))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(
        skills["skills"][0]["metadata"]["id"],
        "example/fixture-skill"
    );
    assert_eq!(skills["skills"][0]["state"]["enabled"], false);

    let readiness = json_response(
        app.clone()
            .oneshot(request(
                "GET",
                &format!("/admin/nexara/skills/{skill_id}/readiness"),
                Some("secret"),
                None,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(readiness["ready"], false);
    assert_eq!(readiness["missing_secrets"], json!(["API_TOKEN"]));

    let enabled = json_response(
        app.clone()
            .oneshot(request(
                "POST",
                &format!("/admin/nexara/skills/{skill_id}/enable"),
                Some("secret"),
                None,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(enabled["enabled"], true);

    let secret = json_response(
        app.clone()
            .oneshot(request(
                "POST",
                &format!("/admin/nexara/skills/{skill_id}/secrets/API_TOKEN"),
                Some("secret"),
                Some(json!({ "value": "token-value" })),
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(secret["status"], "ok");

    let secrets = json_response(
        app.clone()
            .oneshot(request(
                "GET",
                &format!("/admin/nexara/skills/{skill_id}/secrets"),
                Some("secret"),
                None,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(secrets["secrets"][0]["configured"], true);

    let ready = json_response(
        app.clone()
            .oneshot(request(
                "GET",
                &format!("/admin/nexara/skills/{skill_id}/readiness"),
                Some("secret"),
                None,
            ))
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(ready["ready"], true);

    let disabled = json_response(
        app.oneshot(request(
            "POST",
            &format!("/admin/nexara/skills/{skill_id}/disable"),
            Some("secret"),
            None,
        ))
        .await
        .unwrap(),
    )
    .await;
    assert_eq!(disabled["enabled"], false);
}

#[tokio::test]
async fn admin_rejects_undeclared_secret() {
    let app = test_router();
    let skill_id = "example%2Ffixture-skill";
    let _ = app
        .clone()
        .oneshot(request(
            "POST",
            "/admin/nexara/skills/install",
            Some("secret"),
            Some(json!({ "manifest_toml": MANIFEST, "enabled": true })),
        ))
        .await
        .unwrap();

    let response = app
        .oneshot(request(
            "POST",
            &format!("/admin/nexara/skills/{skill_id}/secrets/OTHER_TOKEN"),
            Some("secret"),
            Some(json!({ "value": "token-value" })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = body_json(response).await;
    assert_eq!(body["error"], "secret is not declared by this skill");
}

fn test_router() -> axum::Router {
    let dir = tempfile::tempdir().unwrap().keep();
    let runtime = NexaraRuntime::new(
        NexaraRuntimeConfig::default(),
        Arc::new(StaticToolCatalog::new(vec![ToolDescriptor::read_only(
            "echo",
            "Echo request params",
        )])),
        Arc::new(EchoExecutor),
        Arc::new(StaticTrustPolicyResolver::new(TrustProfile::Observe)),
    );
    router(NexaraServerState {
        runtime: Arc::new(runtime),
        auth: ServerAuth::require("secret"),
        installed_skills: default_installed_store(dir.join("skills.json")),
        secrets: memory_secret_store(),
        metrics: Arc::new(ServerMetrics::default()),
    })
}

fn request(
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<serde_json::Value>,
) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(match body {
            Some(value) => Body::from(serde_json::to_vec(&value).unwrap()),
            None => Body::empty(),
        })
        .unwrap()
}

async fn json_response(response: axum::response::Response) -> serde_json::Value {
    assert!(
        response.status().is_success(),
        "status: {}",
        response.status()
    );
    body_json(response).await
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

struct EchoExecutor;

#[async_trait]
impl HostToolExecutor<()> for EchoExecutor {
    async fn call_host_tool(
        &self,
        _tool: &ToolDescriptor,
        request: ToolCallRequest,
        _context: (),
    ) -> Result<ToolCallResult, nexara_core::NexaraError> {
        Ok(ToolCallResult::new(json!({ "echo": request.params })))
    }
}
