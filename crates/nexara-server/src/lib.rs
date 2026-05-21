use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::get, routing::post};
use nexara_core::{ToolCallRequest, ToolCallResult};
use nexara_registry::installed::InstalledSkillStore;
use nexara_registry::{
    InstalledSkillRecord, InstalledSkillState, JsonFileInstalledSkillStore,
    parse_skill_manifest_str,
};
use nexara_runtime::NexaraRuntime;
use nexara_secrets::{MemorySecretStore, SecretStore, SecretValue};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use subtle::ConstantTimeEq;

#[derive(Clone)]
pub struct NexaraServerState {
    pub runtime: Arc<NexaraRuntime<()>>,
    pub auth: ServerAuth,
    pub installed_skills: JsonFileInstalledSkillStore,
    pub secrets: Arc<dyn SecretStore>,
    pub metrics: Arc<ServerMetrics>,
}

#[derive(Debug, Default)]
pub struct ServerMetrics {
    pub tool_list_requests: AtomicU64,
    pub tool_call_requests: AtomicU64,
    pub admin_requests: AtomicU64,
    pub unauthorized_requests: AtomicU64,
}

#[derive(Clone, Default)]
pub struct ServerAuth {
    pub bearer_token: Option<String>,
    pub allow_dev_no_auth: bool,
    pub authorizer: Option<Arc<dyn Authorizer>>,
}

impl std::fmt::Debug for ServerAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerAuth")
            .field(
                "bearer_token",
                &self.bearer_token.as_ref().map(|_| "[redacted]"),
            )
            .field("allow_dev_no_auth", &self.allow_dev_no_auth)
            .field("authorizer", &self.authorizer.as_ref().map(|_| "custom"))
            .finish()
    }
}

pub trait Authorizer: Send + Sync {
    fn authorize(&self, headers: &HeaderMap) -> bool;
}

impl ServerAuth {
    pub fn require(token: impl Into<String>) -> Self {
        Self {
            bearer_token: Some(token.into()),
            allow_dev_no_auth: false,
            authorizer: None,
        }
    }

    pub fn with_authorizer(authorizer: Arc<dyn Authorizer>) -> Self {
        Self {
            bearer_token: None,
            allow_dev_no_auth: false,
            authorizer: Some(authorizer),
        }
    }

    pub fn authorize(&self, headers: &HeaderMap) -> bool {
        if let Some(authorizer) = &self.authorizer {
            return authorizer.authorize(headers);
        }
        if self.allow_dev_no_auth {
            return true;
        }
        let Some(expected) = &self.bearer_token else {
            return false;
        };
        let Some(raw) = headers.get(axum::http::header::AUTHORIZATION) else {
            return false;
        };
        let Ok(raw) = raw.to_str() else {
            return false;
        };
        let actual = raw.strip_prefix("Bearer ").unwrap_or(raw);
        actual.as_bytes().ct_eq(expected.as_bytes()).into()
    }
}

pub fn router(state: NexaraServerState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/nexara/tools", get(list_tools))
        .route("/v1/nexara/tools/call", post(call_tool))
        .route("/admin/nexara/skills", get(admin_list_skills))
        .route("/admin/nexara/skills/install", post(admin_install_skill))
        .route(
            "/admin/nexara/skills/:skill_id/enable",
            post(admin_enable_skill),
        )
        .route(
            "/admin/nexara/skills/:skill_id/disable",
            post(admin_disable_skill),
        )
        .route(
            "/admin/nexara/skills/:skill_id/readiness",
            get(admin_skill_readiness),
        )
        .route(
            "/admin/nexara/skills/:skill_id/secrets",
            get(admin_skill_secrets),
        )
        .route(
            "/admin/nexara/skills/:skill_id/secrets/:secret_name",
            post(admin_put_skill_secret).delete(admin_delete_skill_secret),
        )
        .route("/metrics", get(metrics))
        .with_state(Arc::new(state))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

#[derive(Debug, Deserialize, Default)]
struct ToolsQuery {
    prompt: Option<String>,
}

async fn list_tools(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Query(query): Query<ToolsQuery>,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state
        .metrics
        .tool_list_requests
        .fetch_add(1, Ordering::Relaxed);
    let scopes = vec!["read".to_string()];
    let tools = state.runtime.list_tools(query.prompt.as_deref(), &scopes);
    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "tools": tools })),
    )
        .into_response()
}

async fn call_tool(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Json(request): Json<ToolCallRequest>,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state
        .metrics
        .tool_call_requests
        .fetch_add(1, Ordering::Relaxed);
    match state.runtime.call_tool(request, ()).await {
        Ok(ToolCallResult { result }) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "result": result })),
        )
            .into_response(),
        Err(err) => error_response(err),
    }
}

async fn admin_list_skills(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state.metrics.admin_requests.fetch_add(1, Ordering::Relaxed);
    match state.installed_skills.list() {
        Ok(skills) => (
            StatusCode::OK,
            Json(json!({ "status": "ok", "skills": skills })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct InstallSkillRequest {
    manifest_toml: String,
    #[serde(default)]
    enabled: bool,
}

async fn admin_install_skill(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Json(request): Json<InstallSkillRequest>,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state.metrics.admin_requests.fetch_add(1, Ordering::Relaxed);
    let metadata = match parse_skill_manifest_str(&request.manifest_toml) {
        Ok(metadata) => metadata,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response();
        }
    };
    let record = InstalledSkillRecord {
        metadata,
        state: InstalledSkillState {
            installed: true,
            enabled: request.enabled,
        },
    };
    if let Err(err) = state.installed_skills.put(record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response();
    }
    (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()
}

async fn admin_enable_skill(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Path(skill_id): Path<String>,
) -> Response {
    update_skill_enabled(state, headers, skill_id, true)
}

async fn admin_disable_skill(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Path(skill_id): Path<String>,
) -> Response {
    update_skill_enabled(state, headers, skill_id, false)
}

fn update_skill_enabled(
    state: Arc<NexaraServerState>,
    headers: HeaderMap,
    skill_id: String,
    enabled: bool,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state.metrics.admin_requests.fetch_add(1, Ordering::Relaxed);
    let Some(mut record) = (match state.installed_skills.get(&skill_id) {
        Ok(record) => record,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response();
        }
    }) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "skill not found" })),
        )
            .into_response();
    };
    record.state.enabled = enabled;
    if let Err(err) = state.installed_skills.put(record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "enabled": enabled })),
    )
        .into_response()
}

async fn admin_skill_readiness(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Path(skill_id): Path<String>,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state.metrics.admin_requests.fetch_add(1, Ordering::Relaxed);
    let Some(record) = (match state.installed_skills.get(&skill_id) {
        Ok(record) => record,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response();
        }
    }) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "skill not found" })),
        )
            .into_response();
    };
    let mut missing = Vec::new();
    let mut present = Vec::new();
    for secret in &record.metadata.requires_secrets {
        match state.secrets.get(secret) {
            Ok(Some(_)) => present.push(secret.clone()),
            Ok(None) => missing.push(secret.clone()),
            Err(err) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": err.to_string() })),
                )
                    .into_response();
            }
        }
    }
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "ready": missing.is_empty(),
            "installed": record.state.installed,
            "enabled": record.state.enabled,
            "present_secrets": present,
            "missing_secrets": missing
        })),
    )
        .into_response()
}

async fn admin_skill_secrets(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Path(skill_id): Path<String>,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state.metrics.admin_requests.fetch_add(1, Ordering::Relaxed);
    let Some(record) = (match state.installed_skills.get(&skill_id) {
        Ok(record) => record,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response();
        }
    }) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "skill not found" })),
        )
            .into_response();
    };
    let statuses = record
        .metadata
        .requires_secrets
        .iter()
        .map(|secret| {
            let configured = state
                .secrets
                .get(secret)
                .map(|value| value.is_some())
                .unwrap_or(false);
            json!({ "name": secret, "configured": configured })
        })
        .collect::<Vec<_>>();
    (
        StatusCode::OK,
        Json(json!({ "status": "ok", "secrets": statuses })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
struct PutSecretRequest {
    value: String,
}

async fn admin_put_skill_secret(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Path((skill_id, secret_name)): Path<(String, String)>,
    Json(request): Json<PutSecretRequest>,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state.metrics.admin_requests.fetch_add(1, Ordering::Relaxed);
    let Some(record) = (match state.installed_skills.get(&skill_id) {
        Ok(record) => record,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err.to_string() })),
            )
                .into_response();
        }
    }) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "skill not found" })),
        )
            .into_response();
    };
    if !record.metadata.requires_secrets.contains(&secret_name) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "secret is not declared by this skill" })),
        )
            .into_response();
    }
    if let Err(err) = state
        .secrets
        .put(&secret_name, SecretValue::new(request.value))
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response();
    }
    (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()
}

async fn admin_delete_skill_secret(
    State(state): State<Arc<NexaraServerState>>,
    headers: HeaderMap,
    Path((skill_id, secret_name)): Path<(String, String)>,
) -> Response {
    if !state.auth.authorize(&headers) {
        return unauthorized(&state);
    }
    state.metrics.admin_requests.fetch_add(1, Ordering::Relaxed);
    if matches!(state.installed_skills.get(&skill_id), Ok(None)) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "skill not found" })),
        )
            .into_response();
    }
    if let Err(err) = state.secrets.delete(&secret_name) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err.to_string() })),
        )
            .into_response();
    }
    (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response()
}

async fn metrics(State(state): State<Arc<NexaraServerState>>) -> Json<serde_json::Value> {
    Json(json!({
        "tool_list_requests": state.metrics.tool_list_requests.load(Ordering::Relaxed),
        "tool_call_requests": state.metrics.tool_call_requests.load(Ordering::Relaxed),
        "admin_requests": state.metrics.admin_requests.load(Ordering::Relaxed),
        "unauthorized_requests": state.metrics.unauthorized_requests.load(Ordering::Relaxed)
    }))
}

fn unauthorized(state: &NexaraServerState) -> Response {
    state
        .metrics
        .unauthorized_requests
        .fetch_add(1, Ordering::Relaxed);
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({ "error": "unauthorized" })),
    )
        .into_response()
}

fn error_response(err: nexara_core::NexaraError) -> Response {
    let status = match err {
        nexara_core::NexaraError::ToolNotFound => StatusCode::NOT_FOUND,
        nexara_core::NexaraError::ToolNotAllowed => StatusCode::FORBIDDEN,
        nexara_core::NexaraError::TrustPolicyDenied(_) => StatusCode::FORBIDDEN,
        nexara_core::NexaraError::ConfirmationRequired(_) => StatusCode::CONFLICT,
        nexara_core::NexaraError::InvalidParams(_) => StatusCode::BAD_REQUEST,
        nexara_core::NexaraError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        nexara_core::NexaraError::ExecutionFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
        nexara_core::NexaraError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
        nexara_core::NexaraError::ConcurrencyLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
        nexara_core::NexaraError::InvalidDescriptor(_) => StatusCode::BAD_REQUEST,
    };
    (status, Json(json!({ "error": err.to_string() }))).into_response()
}

pub fn default_installed_store(path: impl Into<PathBuf>) -> JsonFileInstalledSkillStore {
    JsonFileInstalledSkillStore::new(path)
}

pub fn memory_secret_store() -> Arc<dyn SecretStore> {
    Arc::new(MemorySecretStore::default())
}
