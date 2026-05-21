use nexara_core::{ToolDescriptor, ToolSelectionRequest, ToolUsageSignalProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolSelectionContext {
    pub tenant_id: String,
    pub user_id: String,
    pub surface: String,
}

impl Default for ToolSelectionContext {
    fn default() -> Self {
        Self {
            tenant_id: "default".to_string(),
            user_id: "anonymous".to_string(),
            surface: "unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolUsageEventKind {
    Exposure,
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolUsageEvent {
    pub tool_name: String,
    pub context: ToolSelectionContext,
    pub kind: ToolUsageEventKind,
    pub timestamp_unix_ms: u64,
    #[serde(default)]
    pub latency_ms: Option<u64>,
}

pub trait ToolUsageSignalStore: Send + Sync {
    fn record(&self, event: ToolUsageEvent) -> anyhow::Result<()>;
    fn events_for_tool(&self, tool_name: &str) -> anyhow::Result<Vec<ToolUsageEvent>>;
}

#[derive(Debug, Clone)]
pub struct LearningScorer {
    max_adjustment: f32,
    decay: DecayConfig,
}

impl Default for LearningScorer {
    fn default() -> Self {
        Self {
            max_adjustment: 2.0,
            decay: DecayConfig::default(),
        }
    }
}

impl LearningScorer {
    pub fn new(max_adjustment: f32) -> Self {
        Self {
            max_adjustment: max_adjustment.max(0.0),
            decay: DecayConfig::default(),
        }
    }

    pub fn with_decay(mut self, decay: DecayConfig) -> Self {
        self.decay = decay;
        self
    }

    pub fn score_events(&self, events: &[ToolUsageEvent]) -> f32 {
        self.explain_events(events).adjustment
    }

    pub fn explain_events(&self, events: &[ToolUsageEvent]) -> LearningScoreExplanation {
        let mut score = 0.0_f32;
        let now = events
            .iter()
            .map(|event| event.timestamp_unix_ms)
            .max()
            .unwrap_or_default();
        let mut considered_events = 0_usize;
        for event in events.iter().rev().take(64) {
            considered_events += 1;
            let weight = self.decay.weight(now, event.timestamp_unix_ms);
            let delta = match event.kind {
                ToolUsageEventKind::Exposure => 0.02,
                ToolUsageEventKind::Success => 0.25,
                ToolUsageEventKind::Failure => -0.35,
            };
            score += delta * weight;
        }
        LearningScoreExplanation {
            adjustment: score.clamp(-self.max_adjustment, self.max_adjustment),
            max_adjustment: self.max_adjustment,
            considered_events,
            decay_half_life_ms: self.decay.half_life_ms,
            reason: "recent exposures and outcomes within bounded influence".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecayConfig {
    pub half_life_ms: Option<u64>,
}

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            half_life_ms: Some(24 * 60 * 60 * 1000),
        }
    }
}

impl DecayConfig {
    fn weight(&self, now_ms: u64, event_ms: u64) -> f32 {
        let Some(half_life_ms) = self.half_life_ms else {
            return 1.0;
        };
        if half_life_ms == 0 || now_ms <= event_ms {
            return 1.0;
        }
        let elapsed = (now_ms - event_ms) as f64;
        0.5_f64.powf(elapsed / half_life_ms as f64) as f32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningScoreExplanation {
    pub adjustment: f32,
    pub max_adjustment: f32,
    pub considered_events: usize,
    pub decay_half_life_ms: Option<u64>,
    pub reason: String,
}

pub struct LearningSignalProvider<S> {
    store: S,
    scorer: LearningScorer,
}

impl<S> LearningSignalProvider<S> {
    pub fn new(store: S, scorer: LearningScorer) -> Self {
        Self { store, scorer }
    }

    pub fn explain_tool(&self, tool_name: &str) -> anyhow::Result<LearningScoreExplanation>
    where
        S: ToolUsageSignalStore,
    {
        let events = self.store.events_for_tool(tool_name)?;
        Ok(self.scorer.explain_events(&events))
    }
}

impl<S> ToolUsageSignalProvider for LearningSignalProvider<S>
where
    S: ToolUsageSignalStore,
{
    fn score_adjustment(&self, tool: &ToolDescriptor, _request: &ToolSelectionRequest) -> f32 {
        self.store
            .events_for_tool(&tool.name)
            .map(|events| self.scorer.score_events(&events))
            .unwrap_or(0.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InProcessUsageSignalStore {
    events: std::sync::Arc<std::sync::Mutex<HashMap<String, Vec<ToolUsageEvent>>>>,
}

impl ToolUsageSignalStore for InProcessUsageSignalStore {
    fn record(&self, event: ToolUsageEvent) -> anyhow::Result<()> {
        self.events
            .lock()
            .unwrap()
            .entry(event.tool_name.clone())
            .or_default()
            .push(event);
        Ok(())
    }

    fn events_for_tool(&self, tool_name: &str) -> anyhow::Result<Vec<ToolUsageEvent>> {
        Ok(self
            .events
            .lock()
            .unwrap()
            .get(tool_name)
            .cloned()
            .unwrap_or_default())
    }
}
