use crate::policy::TrustTier;
use crate::registry::ToolRegistry;
use crate::tool::{ToolDescriptor, ToolSelectionRequest};
use std::collections::HashSet;

pub trait ToolUsageSignalProvider: Send + Sync {
    fn score_adjustment(&self, tool: &ToolDescriptor, request: &ToolSelectionRequest) -> f32;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrokerScoredTool {
    pub descriptor: ToolDescriptor,
    pub base_score: f32,
    pub learned_adjustment: f32,
    pub final_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToolSelectionExplanation {
    pub scored_tools: Vec<BrokerScoredTool>,
}

#[derive(Default)]
pub struct ToolBroker {
    usage_signals: Option<Box<dyn ToolUsageSignalProvider>>,
    max_learned_adjustment: f32,
}

impl std::fmt::Debug for ToolBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolBroker")
            .field(
                "usage_signals",
                &self.usage_signals.as_ref().map(|_| "enabled"),
            )
            .field("max_learned_adjustment", &self.max_learned_adjustment)
            .finish()
    }
}

impl ToolBroker {
    pub fn new() -> Self {
        Self {
            usage_signals: None,
            max_learned_adjustment: 2.0,
        }
    }

    pub fn with_usage_signals(
        mut self,
        usage_signals: Box<dyn ToolUsageSignalProvider>,
        max_adjustment: f32,
    ) -> Self {
        self.usage_signals = Some(usage_signals);
        self.max_learned_adjustment = max_adjustment.max(0.0);
        self
    }

    pub fn score_tool_descriptor(tool: &ToolDescriptor, prompt: &str) -> f32 {
        let prompt = prompt.to_lowercase();
        let mut score = 0.0_f32;
        for tag in &tool.tags {
            if prompt.contains(&tag.to_lowercase()) {
                score += 3.0;
            }
        }
        if prompt.contains(&tool.name.replace('_', " ")) || prompt.contains(&tool.name) {
            score += 4.0;
        }
        if tool.name == "run_consensus" && !prompt.contains("status") {
            score += 1.0;
        }
        if matches!(tool.trust_tier, TrustTier::Builtin) {
            score += 1.0;
        }
        score
    }

    pub fn select(
        &self,
        registry: &ToolRegistry,
        request: &ToolSelectionRequest,
    ) -> Vec<ToolDescriptor> {
        self.select_with_explanation(registry, request)
            .0
            .into_iter()
            .map(|scored| scored.descriptor)
            .collect()
    }

    pub fn select_with_explanation(
        &self,
        registry: &ToolRegistry,
        request: &ToolSelectionRequest,
    ) -> (Vec<BrokerScoredTool>, ToolSelectionExplanation) {
        let prompt = request.prompt.to_lowercase();
        let granted_scopes: HashSet<&str> = request.scopes.iter().map(String::as_str).collect();

        let mut scored = registry
            .all()
            .into_iter()
            .filter(|tool| {
                tool.enabled
                    && !matches!(tool.trust_tier, TrustTier::LocalExternalProcess)
                    && Self::scope_allowed(tool, &granted_scopes)
            })
            .map(|tool| {
                let base_score = Self::score_tool_descriptor(&tool, &prompt);
                let learned_adjustment = self
                    .usage_signals
                    .as_ref()
                    .map(|signals| signals.score_adjustment(&tool, request))
                    .unwrap_or(0.0)
                    .clamp(-self.max_learned_adjustment, self.max_learned_adjustment);
                let final_score = base_score + learned_adjustment;
                BrokerScoredTool {
                    descriptor: tool,
                    base_score,
                    learned_adjustment,
                    final_score,
                }
            })
            .collect::<Vec<_>>();

        scored.sort_by(|left, right| {
            right
                .final_score
                .partial_cmp(&left.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.descriptor.name.cmp(&right.descriptor.name))
        });

        let explanation = ToolSelectionExplanation {
            scored_tools: scored.clone(),
        };
        let selected = scored.into_iter().take(request.max_tools).collect();
        (selected, explanation)
    }

    fn scope_allowed(tool: &ToolDescriptor, granted_scopes: &HashSet<&str>) -> bool {
        tool.scopes
            .iter()
            .all(|scope| granted_scopes.contains(scope.as_str()))
    }
}
