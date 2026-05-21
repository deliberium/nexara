use async_trait::async_trait;
use nexara_core::{ToolCallRequest, ToolCallResult, ToolDescriptor, TrustTier};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteEndpoint {
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDiscoveryRequest {
    pub endpoint: RemoteEndpoint,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDiscoveryResponse {
    pub tools: Vec<ToolDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteToolDiagnostic {
    pub endpoint: String,
    pub error: String,
    pub circuit_open: bool,
}

#[async_trait]
pub trait ToolDiscovery: Send + Sync {
    async fn list_tools(
        &self,
        request: ToolDiscoveryRequest,
    ) -> anyhow::Result<ToolDiscoveryResponse>;
}

#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn call_tool(&self, request: ToolCallRequest) -> anyhow::Result<ToolCallResult>;
}

#[async_trait]
pub trait RemoteAuthProvider: Send + Sync {
    async fn auth_header_for(&self, endpoint: &RemoteEndpoint) -> anyhow::Result<Option<String>>;
}

#[derive(Debug, Clone, Default)]
pub struct StaticBearerAuthProvider {
    token: Option<String>,
}

impl StaticBearerAuthProvider {
    pub fn new(token: Option<String>) -> Self {
        Self { token }
    }
}

#[async_trait]
impl RemoteAuthProvider for StaticBearerAuthProvider {
    async fn auth_header_for(&self, _endpoint: &RemoteEndpoint) -> anyhow::Result<Option<String>> {
        Ok(self.token.as_ref().map(|token| format!("Bearer {token}")))
    }
}

#[derive(Clone)]
pub struct HttpRemoteToolClient {
    client: reqwest::Client,
    allowed_hosts: Vec<String>,
    auth: Arc<dyn RemoteAuthProvider>,
    circuit_breakers: Arc<Mutex<HashMap<String, CircuitBreakerState>>>,
    tool_routes: Arc<Mutex<HashMap<String, RemoteEndpoint>>>,
}

#[derive(Debug, Clone, Default)]
struct CircuitBreakerState {
    consecutive_failures: u32,
    open_until: Option<Instant>,
}

impl HttpRemoteToolClient {
    pub fn new(
        allowed_hosts: Vec<String>,
        auth: Arc<dyn RemoteAuthProvider>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(3))
                .timeout(Duration::from_secs(10))
                .build()?,
            allowed_hosts,
            auth,
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
            tool_routes: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn validate_endpoint(&self, endpoint: &RemoteEndpoint) -> anyhow::Result<()> {
        let parsed = reqwest::Url::parse(&endpoint.base_url)?;
        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("missing remote host"))?;
        if self.allowed_hosts.is_empty() || self.allowed_hosts.iter().any(|allowed| allowed == host)
        {
            Ok(())
        } else {
            anyhow::bail!("host '{host}' is not allowlisted")
        }
    }

    async fn auth_request(
        &self,
        request: reqwest::RequestBuilder,
        endpoint: &RemoteEndpoint,
    ) -> anyhow::Result<reqwest::RequestBuilder> {
        if let Some(header) = self.auth.auth_header_for(endpoint).await? {
            Ok(request.header(reqwest::header::AUTHORIZATION, header))
        } else {
            Ok(request)
        }
    }

    async fn is_circuit_open(&self, endpoint: &RemoteEndpoint) -> bool {
        let breakers = self.circuit_breakers.lock().await;
        breakers
            .get(&endpoint.base_url)
            .and_then(|state| state.open_until)
            .is_some_and(|until| until > Instant::now())
    }

    async fn mark_failure(&self, endpoint: &RemoteEndpoint) {
        let mut breakers = self.circuit_breakers.lock().await;
        let state = breakers.entry(endpoint.base_url.clone()).or_default();
        state.consecutive_failures += 1;
        if state.consecutive_failures >= 3 {
            state.open_until = Some(Instant::now() + Duration::from_secs(30));
        }
    }

    async fn mark_success(&self, endpoint: &RemoteEndpoint) {
        self.circuit_breakers
            .lock()
            .await
            .remove(&endpoint.base_url);
    }
}

#[async_trait]
impl ToolDiscovery for HttpRemoteToolClient {
    async fn list_tools(
        &self,
        request: ToolDiscoveryRequest,
    ) -> anyhow::Result<ToolDiscoveryResponse> {
        self.validate_endpoint(&request.endpoint)?;
        if self.is_circuit_open(&request.endpoint).await {
            anyhow::bail!("remote circuit breaker is open");
        }
        let mut outbound = self.client.get(format!(
            "{}/v1/nexara/tools",
            request.endpoint.base_url.trim_end_matches('/')
        ));
        if let Some(prompt) = &request.prompt {
            outbound = outbound.query(&[("prompt", prompt)]);
        }
        outbound = self.auth_request(outbound, &request.endpoint).await?;
        let response = match outbound.send().await {
            Ok(response) => response,
            Err(err) => {
                self.mark_failure(&request.endpoint).await;
                return Err(err.into());
            }
        };
        if !response.status().is_success() {
            self.mark_failure(&request.endpoint).await;
            anyhow::bail!("remote returned {}", response.status());
        }
        let payload: serde_json::Value = response.json().await?;
        let mut tools: Vec<ToolDescriptor> =
            serde_json::from_value(payload.get("tools").cloned().unwrap_or_else(|| json!([])))?;
        for tool in &mut tools {
            tool.trust_tier = TrustTier::RemoteNexara;
            self.tool_routes
                .lock()
                .await
                .insert(tool.name.clone(), request.endpoint.clone());
        }
        self.mark_success(&request.endpoint).await;
        Ok(ToolDiscoveryResponse { tools })
    }
}

#[async_trait]
impl ToolExecutor for HttpRemoteToolClient {
    async fn call_tool(&self, request: ToolCallRequest) -> anyhow::Result<ToolCallResult> {
        let endpoint = self
            .tool_routes
            .lock()
            .await
            .get(&request.tool)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("tool route not found"))?;
        self.validate_endpoint(&endpoint)?;
        if self.is_circuit_open(&endpoint).await {
            anyhow::bail!("remote circuit breaker is open");
        }
        let outbound = self.client.post(format!(
            "{}/v1/nexara/tools/call",
            endpoint.base_url.trim_end_matches('/')
        ));
        let outbound = self.auth_request(outbound, &endpoint).await?;
        let response = outbound.json(&request).send().await?;
        if !response.status().is_success() {
            self.mark_failure(&endpoint).await;
            anyhow::bail!("remote returned {}", response.status());
        }
        let payload: serde_json::Value = response.json().await?;
        self.mark_success(&endpoint).await;
        Ok(ToolCallResult::new(
            payload.get("result").cloned().unwrap_or_else(|| json!({})),
        ))
    }
}
