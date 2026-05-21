use async_trait::async_trait;
use nexara_core::{
    ActionClass, AuditOutcome, AuditRecord, AuditSink, EffectiveTrustPolicy, NexaraError,
    PolicyDecision, ToolBroker, ToolCallRequest, ToolCallResult, ToolDescriptor, ToolRegistry,
    ToolSelectionRequest, ToolUsageSignalProvider, TracingAuditSink, TrustProfile, hash_result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NexaraRuntimeConfig {
    pub max_tools_per_request: usize,
    pub max_payload_bytes: usize,
    pub max_concurrent_calls: usize,
    pub default_scopes: Vec<String>,
    pub tool_selection_enabled: bool,
}

impl Default for NexaraRuntimeConfig {
    fn default() -> Self {
        Self {
            max_tools_per_request: 8,
            max_payload_bytes: 64 * 1024,
            max_concurrent_calls: 8,
            default_scopes: vec!["read".to_string()],
            tool_selection_enabled: true,
        }
    }
}

pub trait ToolCatalogProvider: Send + Sync {
    fn tools(&self) -> Vec<ToolDescriptor>;
}

#[derive(Default)]
pub struct CompositeToolCatalog {
    providers: Vec<Arc<dyn ToolCatalogProvider>>,
}

impl CompositeToolCatalog {
    pub fn new(providers: Vec<Arc<dyn ToolCatalogProvider>>) -> Self {
        Self { providers }
    }
}

impl ToolCatalogProvider for CompositeToolCatalog {
    fn tools(&self) -> Vec<ToolDescriptor> {
        self.providers
            .iter()
            .flat_map(|provider| provider.tools())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct StaticToolCatalog {
    tools: Vec<ToolDescriptor>,
}

impl StaticToolCatalog {
    pub fn new(tools: Vec<ToolDescriptor>) -> Self {
        Self { tools }
    }
}

impl ToolCatalogProvider for StaticToolCatalog {
    fn tools(&self) -> Vec<ToolDescriptor> {
        self.tools.clone()
    }
}

#[async_trait]
pub trait HostToolExecutor<C>: Send + Sync {
    async fn call_host_tool(
        &self,
        tool: &ToolDescriptor,
        request: ToolCallRequest,
        context: C,
    ) -> Result<ToolCallResult, NexaraError>;
}

pub trait TrustPolicyResolver<C>: Send + Sync {
    fn resolve(&self, context: &C) -> EffectiveTrustPolicy;
}

#[async_trait]
pub trait RuntimeLifecycleHooks<C>: Send + Sync {
    async fn before_call(&self, _tool: &ToolDescriptor, _request: &ToolCallRequest, _context: &C) {}

    async fn after_call(
        &self,
        _tool: &ToolDescriptor,
        _request: &ToolCallRequest,
        _outcome: Result<&ToolCallResult, &NexaraError>,
        _context: &C,
    ) {
    }
}

#[derive(Debug, Default)]
pub struct NoopRuntimeLifecycleHooks;

#[async_trait]
impl<C> RuntimeLifecycleHooks<C> for NoopRuntimeLifecycleHooks where C: Send + Sync {}

#[derive(Debug, Clone)]
pub struct StaticTrustPolicyResolver {
    profile: TrustProfile,
}

impl StaticTrustPolicyResolver {
    pub fn new(profile: TrustProfile) -> Self {
        Self { profile }
    }
}

impl Default for StaticTrustPolicyResolver {
    fn default() -> Self {
        Self {
            profile: TrustProfile::Observe,
        }
    }
}

impl<C> TrustPolicyResolver<C> for StaticTrustPolicyResolver {
    fn resolve(&self, _context: &C) -> EffectiveTrustPolicy {
        EffectiveTrustPolicy::for_profile(self.profile)
    }
}

pub struct NexaraRuntime<C> {
    config: NexaraRuntimeConfig,
    catalog: Arc<dyn ToolCatalogProvider>,
    executor: Arc<dyn HostToolExecutor<C>>,
    trust_resolver: Arc<dyn TrustPolicyResolver<C>>,
    audit_sink: Arc<dyn AuditSink>,
    lifecycle_hooks: Arc<dyn RuntimeLifecycleHooks<C>>,
    broker: ToolBroker,
    call_semaphore: Arc<Semaphore>,
}

impl<C> NexaraRuntime<C>
where
    C: Clone + Send + Sync + 'static,
{
    pub fn new(
        config: NexaraRuntimeConfig,
        catalog: Arc<dyn ToolCatalogProvider>,
        executor: Arc<dyn HostToolExecutor<C>>,
        trust_resolver: Arc<dyn TrustPolicyResolver<C>>,
    ) -> Self {
        let max_concurrent_calls = config.max_concurrent_calls;
        Self {
            config,
            catalog,
            executor,
            trust_resolver,
            audit_sink: Arc::new(TracingAuditSink),
            lifecycle_hooks: Arc::new(NoopRuntimeLifecycleHooks),
            broker: ToolBroker::new(),
            call_semaphore: Arc::new(Semaphore::new(max_concurrent_calls)),
        }
    }

    pub fn with_audit_sink(mut self, audit_sink: Arc<dyn AuditSink>) -> Self {
        self.audit_sink = audit_sink;
        self
    }

    pub fn with_broker(mut self, broker: ToolBroker) -> Self {
        self.broker = broker;
        self
    }

    pub fn with_usage_signals(
        mut self,
        usage_signals: Box<dyn ToolUsageSignalProvider>,
        max_adjustment: f32,
    ) -> Self {
        self.broker = self
            .broker
            .with_usage_signals(usage_signals, max_adjustment);
        self
    }

    pub fn with_lifecycle_hooks(mut self, hooks: Arc<dyn RuntimeLifecycleHooks<C>>) -> Self {
        self.lifecycle_hooks = hooks;
        self
    }

    pub fn registry(&self) -> ToolRegistry {
        let mut registry = ToolRegistry::new();
        for tool in self.catalog.tools() {
            registry.register(tool);
        }
        registry
    }

    pub fn list_tools(&self, prompt: Option<&str>, scopes: &[String]) -> Vec<ToolDescriptor> {
        let registry = self.registry();
        if self.config.tool_selection_enabled && prompt.is_some() {
            self.broker.select(
                &registry,
                &ToolSelectionRequest {
                    prompt: prompt.unwrap_or_default().to_string(),
                    scopes: scopes.to_vec(),
                    max_tools: self.config.max_tools_per_request,
                },
            )
        } else {
            registry
                .all()
                .into_iter()
                .filter(|tool| tool.enabled)
                .take(self.config.max_tools_per_request)
                .collect()
        }
    }

    pub async fn call_tool(
        &self,
        mut request: ToolCallRequest,
        context: C,
    ) -> Result<ToolCallResult, NexaraError> {
        let started = Instant::now();
        let payload_size = serde_json::to_vec(&request.params)
            .map(|payload| payload.len())
            .unwrap_or(usize::MAX);
        if payload_size > self.config.max_payload_bytes {
            return Err(NexaraError::PayloadTooLarge(payload_size));
        }
        let registry = self.registry();
        let tool = registry
            .get(&request.tool)
            .cloned()
            .ok_or(NexaraError::ToolNotFound)?;
        let policy = self.trust_resolver.resolve(&context);
        self.enforce_policy(&tool, &mut request, &policy)?;
        let permit = self
            .call_semaphore
            .try_acquire()
            .map_err(|_| NexaraError::ConcurrencyLimitExceeded)?;
        self.lifecycle_hooks
            .before_call(&tool, &request, &context)
            .await;
        let result = self
            .executor
            .call_host_tool(&tool, request.clone(), context.clone())
            .await;
        drop(permit);

        let (decision, outcome, hash) = match &result {
            Ok(result) => (
                PolicyDecision::Allowed,
                AuditOutcome::Success,
                Some(hash_result(&result.result)),
            ),
            Err(_) => (PolicyDecision::Allowed, AuditOutcome::Error, None),
        };
        let mut record = AuditRecord::now("unknown", tool.name.clone(), decision, outcome);
        record.duration_ms = started.elapsed().as_millis() as u64;
        record.result_hash = hash;
        let _ = self.audit_sink.record(record);
        self.lifecycle_hooks
            .after_call(&tool, &request, result.as_ref(), &context)
            .await;
        result
    }

    fn enforce_policy(
        &self,
        tool: &ToolDescriptor,
        request: &mut ToolCallRequest,
        policy: &EffectiveTrustPolicy,
    ) -> Result<(), NexaraError> {
        let confirmed = request
            .params
            .get("_confirmed")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        if let Some(object) = request.params.as_object_mut() {
            object.remove("_confirmed");
        }
        match tool.action_class {
            ActionClass::Read if policy.allow_read => Ok(()),
            ActionClass::Read => Err(NexaraError::TrustPolicyDenied(
                "read tools are disabled by trust policy".to_string(),
            )),
            ActionClass::Write if !policy.allow_write => Err(NexaraError::TrustPolicyDenied(
                "write tools are disabled by trust policy".to_string(),
            )),
            ActionClass::Write if policy.require_confirmation_for_write && !confirmed => Err(
                NexaraError::ConfirmationRequired("write tool requires confirmation".to_string()),
            ),
            ActionClass::Write => Ok(()),
            ActionClass::Execute if !policy.allow_execute => Err(NexaraError::TrustPolicyDenied(
                "execute tools are disabled by trust policy".to_string(),
            )),
            ActionClass::Execute if policy.require_confirmation_for_execute && !confirmed => Err(
                NexaraError::ConfirmationRequired("execute tool requires confirmation".to_string()),
            ),
            ActionClass::Execute => Ok(()),
        }
    }
}
