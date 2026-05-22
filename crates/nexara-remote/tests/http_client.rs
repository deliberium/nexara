use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode};
use axum::{Json, Router, routing::get, routing::post};
use nexara_core::ToolCallRequest;
use nexara_remote::{
    HttpRemoteToolClient, RemoteEndpoint, StaticBearerAuthProvider, ToolDiscovery,
    ToolDiscoveryRequest, ToolExecutor,
};
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::test]
async fn list_tools_sends_auth_prompt_and_marks_remote_trust() {
    let Some((endpoint, _state)) = spawn_remote(false).await else {
        return;
    };
    let client = HttpRemoteToolClient::new(
        vec!["127.0.0.1".to_string()],
        Arc::new(StaticBearerAuthProvider::new(Some("secret".to_string()))),
    )
    .unwrap();

    let response = client
        .list_tools(ToolDiscoveryRequest {
            endpoint,
            prompt: Some("search docs".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(response.tools.len(), 1);
    assert_eq!(response.tools[0].name, "remote_search");
    assert_eq!(
        serde_json::to_value(response.tools[0].trust_tier).unwrap(),
        json!("remote_nexara")
    );
}

#[tokio::test]
async fn call_tool_uses_route_learned_from_discovery() {
    let Some((endpoint, state)) = spawn_remote(false).await else {
        return;
    };
    let client = HttpRemoteToolClient::new(
        vec!["127.0.0.1".to_string()],
        Arc::new(StaticBearerAuthProvider::new(Some("secret".to_string()))),
    )
    .unwrap();

    client
        .list_tools(ToolDiscoveryRequest {
            endpoint,
            prompt: None,
        })
        .await
        .unwrap();
    let result = client
        .call_tool(ToolCallRequest {
            tool: "remote_search".to_string(),
            params: json!({ "q": "nexara" }),
        })
        .await
        .unwrap();

    assert_eq!(result.result["tool"], "remote_search");
    assert_eq!(result.result["params"]["q"], "nexara");
    assert_eq!(state.call_count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn reject_non_allowlisted_remote_host_before_request() {
    let endpoint = RemoteEndpoint {
        base_url: "http://127.0.0.1:9".to_string(),
    };
    let client = HttpRemoteToolClient::new(
        vec!["example.com".to_string()],
        Arc::new(StaticBearerAuthProvider::default()),
    )
    .unwrap();

    let err = client
        .list_tools(ToolDiscoveryRequest {
            endpoint,
            prompt: None,
        })
        .await
        .unwrap_err()
        .to_string();

    assert!(err.contains("not allowlisted"));
}

#[tokio::test]
async fn circuit_opens_after_repeated_remote_failures() {
    let Some((endpoint, state)) = spawn_remote(true).await else {
        return;
    };
    let client = HttpRemoteToolClient::new(
        vec!["127.0.0.1".to_string()],
        Arc::new(StaticBearerAuthProvider::default()),
    )
    .unwrap();

    for _ in 0..3 {
        let err = client
            .list_tools(ToolDiscoveryRequest {
                endpoint: endpoint.clone(),
                prompt: None,
            })
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("remote returned"));
    }

    let err = client
        .list_tools(ToolDiscoveryRequest {
            endpoint,
            prompt: None,
        })
        .await
        .unwrap_err()
        .to_string();

    assert!(err.contains("circuit breaker is open"));
    assert_eq!(state.list_count.load(Ordering::Relaxed), 3);
}

#[derive(Default)]
struct RemoteState {
    fail: bool,
    list_count: AtomicUsize,
    call_count: AtomicUsize,
}

async fn spawn_remote(fail: bool) -> Option<(RemoteEndpoint, Arc<RemoteState>)> {
    let state = Arc::new(RemoteState {
        fail,
        ..RemoteState::default()
    });
    let router = Router::new()
        .route("/v1/nexara/tools", get(list_tools))
        .route("/v1/nexara/tools/call", post(call_tool))
        .with_state(state.clone());
    let listener = match TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return None,
        Err(err) => panic!("failed binding remote fixture listener: {err}"),
    };
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    Some((
        RemoteEndpoint {
            base_url: format!("http://{addr}"),
        },
        state,
    ))
}

async fn list_tools(
    axum::extract::State(state): axum::extract::State<Arc<RemoteState>>,
    headers: HeaderMap,
    Query(query): Query<std::collections::BTreeMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    state.list_count.fetch_add(1, Ordering::Relaxed);
    if state.fail {
        return Err(StatusCode::BAD_GATEWAY);
    }
    assert_eq!(
        headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
        Some("Bearer secret")
    );
    if let Some(prompt) = query.get("prompt") {
        assert_eq!(prompt, "search docs");
    }
    Ok(Json(json!({
        "tools": [{
            "name": "remote_search",
            "description": "Search remote docs",
            "tags": ["remote"],
            "scopes": ["read"],
            "trust_tier": "builtin",
            "action_class": "read",
            "enabled": true
        }]
    })))
}

async fn call_tool(
    axum::extract::State(state): axum::extract::State<Arc<RemoteState>>,
    Json(request): Json<ToolCallRequest>,
) -> Json<serde_json::Value> {
    state.call_count.fetch_add(1, Ordering::Relaxed);
    Json(json!({
        "result": {
            "tool": request.tool,
            "params": request.params
        }
    }))
}
