use crate::policy::{ActionClass, TrustTier};
use crate::tool::ToolDescriptor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicySourceKind {
    OneLine,
    Json,
    HostProvided,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicySource {
    pub kind: PolicySourceKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyContract {
    pub id: String,
    pub version: String,
    pub source: PolicySource,
    pub rules: Vec<PolicyRule>,
    pub defaults: PolicyDefaults,
    #[serde(default)]
    pub diagnostics: Vec<PolicyDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDefaults {
    pub deny_by_default: bool,
    pub deny_missing_capabilities: bool,
}

impl Default for PolicyDefaults {
    fn default() -> Self {
        Self {
            deny_by_default: true,
            deny_missing_capabilities: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRule {
    pub id: String,
    pub effect: PolicyEffect,
    pub selector: PolicySelector,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<PolicyCondition>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    Allow,
    Deny,
    RequireConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PolicySelector {
    #[serde(default)]
    pub capabilities: Vec<CapabilityPattern>,
    #[serde(default)]
    pub resources: Vec<ResourcePattern>,
    #[serde(default)]
    pub actions: Vec<ActionPattern>,
    #[serde(default)]
    pub tools: Vec<ToolPattern>,
    #[serde(default)]
    pub scopes: Vec<ScopePattern>,
    #[serde(default)]
    pub trust_tiers: Vec<TrustTier>,
    #[serde(default)]
    pub action_classes: Vec<ActionClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityPattern(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourcePattern(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionPattern(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolPattern(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScopePattern(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_confirmation: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_payload_bytes: Option<usize>,
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trust_tier_at_most: Option<TrustTier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_class_is: Option<ActionClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyDiagnostic {
    pub severity: PolicyDiagnosticSeverity,
    pub code: PolicyDiagnosticCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDiagnosticCode {
    UnknownResource,
    UnknownOperation,
    AmbiguousVerb,
    ConflictingRule,
    OverBroadAllow,
    MissingCatalogMapping,
    DenyOverridesAllow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchedPolicyRule {
    pub rule_id: String,
    pub effect: PolicyEffect,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEvaluationDecision {
    Allowed,
    Denied,
    ConfirmationRequired,
    NoMatchingAllow,
    DescriptorMissingCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyEvaluation {
    pub decision: PolicyEvaluationDecision,
    #[serde(default)]
    pub matched_rules: Vec<MatchedPolicyRule>,
    pub explanation: String,
}

impl PolicyContract {
    pub fn evaluate_tool(&self, tool: &ToolDescriptor, confirmed: bool) -> PolicyEvaluation {
        if self.defaults.deny_missing_capabilities && tool.capabilities.is_empty() {
            return PolicyEvaluation {
                decision: PolicyEvaluationDecision::DescriptorMissingCapabilities,
                matched_rules: Vec::new(),
                explanation: format!(
                    "Denied because '{}' does not declare semantic capability metadata.",
                    tool.name
                ),
            };
        }

        let mut matches = Vec::new();
        for rule in &self.rules {
            if rule.selector.matches_tool(tool) {
                matches.push(MatchedPolicyRule {
                    rule_id: rule.id.clone(),
                    effect: rule.effect,
                    reason: rule.reason.clone(),
                });
            }
        }

        if let Some(rule) = matches
            .iter()
            .find(|rule| matches!(rule.effect, PolicyEffect::Deny))
            .cloned()
        {
            return PolicyEvaluation {
                decision: PolicyEvaluationDecision::Denied,
                matched_rules: matches,
                explanation: format!("Denied by policy rule '{}': {}", rule.rule_id, rule.reason),
            };
        }

        if let Some(rule) = matches
            .iter()
            .find(|rule| matches!(rule.effect, PolicyEffect::RequireConfirmation))
            .cloned()
            && !confirmed
        {
            return PolicyEvaluation {
                decision: PolicyEvaluationDecision::ConfirmationRequired,
                matched_rules: matches,
                explanation: format!(
                    "Confirmation required by policy rule '{}': {}",
                    rule.rule_id, rule.reason
                ),
            };
        }

        if matches
            .iter()
            .any(|rule| matches!(rule.effect, PolicyEffect::Allow))
        {
            return PolicyEvaluation {
                decision: PolicyEvaluationDecision::Allowed,
                matched_rules: matches,
                explanation: format!("Allowed by policy contract '{}'.", self.id),
            };
        }

        if self.defaults.deny_by_default {
            PolicyEvaluation {
                decision: PolicyEvaluationDecision::NoMatchingAllow,
                matched_rules: matches,
                explanation: format!(
                    "Denied because '{}' did not match any allow rule in policy contract '{}'.",
                    tool.name, self.id
                ),
            }
        } else {
            PolicyEvaluation {
                decision: PolicyEvaluationDecision::Allowed,
                matched_rules: matches,
                explanation: format!(
                    "Allowed by policy contract '{}' default allow behavior.",
                    self.id
                ),
            }
        }
    }
}

impl PolicySelector {
    pub fn matches_tool(&self, tool: &ToolDescriptor) -> bool {
        self.matches_tools(tool)
            && self.matches_action_classes(tool)
            && self.matches_trust_tiers(tool)
            && self.matches_scopes(tool)
            && self.matches_capabilities(tool)
            && self.matches_resources(tool)
            && self.matches_actions(tool)
    }

    fn matches_tools(&self, tool: &ToolDescriptor) -> bool {
        self.tools.is_empty()
            || self
                .tools
                .iter()
                .any(|pattern| pattern_matches(&pattern.0, &tool.name))
    }

    fn matches_action_classes(&self, tool: &ToolDescriptor) -> bool {
        self.action_classes.is_empty() || self.action_classes.contains(&tool.action_class)
    }

    fn matches_trust_tiers(&self, tool: &ToolDescriptor) -> bool {
        self.trust_tiers.is_empty() || self.trust_tiers.contains(&tool.trust_tier)
    }

    fn matches_scopes(&self, tool: &ToolDescriptor) -> bool {
        self.scopes.is_empty()
            || self.scopes.iter().any(|pattern| {
                tool.scopes
                    .iter()
                    .any(|scope| pattern_matches(&pattern.0, scope))
            })
    }

    fn matches_capabilities(&self, tool: &ToolDescriptor) -> bool {
        self.capabilities.is_empty()
            || self.capabilities.iter().any(|pattern| {
                tool.capabilities
                    .iter()
                    .any(|capability| pattern_matches(&pattern.0, &capability.id))
            })
    }

    fn matches_resources(&self, tool: &ToolDescriptor) -> bool {
        self.resources.is_empty()
            || self.resources.iter().any(|pattern| {
                tool.capabilities.iter().any(|capability| {
                    pattern_matches(&pattern.0, &capability.resource_path.join("."))
                        || pattern_matches(&pattern.0, &capability.resource)
                })
            })
    }

    fn matches_actions(&self, tool: &ToolDescriptor) -> bool {
        self.actions.is_empty()
            || self.actions.iter().any(|pattern| {
                tool.capabilities
                    .iter()
                    .any(|capability| pattern_matches(&pattern.0, &capability.operation))
            })
    }
}

pub fn pattern_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.trim();
    let value = value.trim();
    if pattern == "*" || pattern == value {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return value == prefix || value.starts_with(&format!("{prefix}."));
    }
    if let Some(suffix) = pattern.strip_prefix("*.") {
        return value == suffix || value.ends_with(&format!(".{suffix}"));
    }
    false
}

pub fn matched_rules_metadata(rules: &[MatchedPolicyRule]) -> BTreeMap<String, String> {
    let mut metadata = BTreeMap::new();
    if !rules.is_empty() {
        metadata.insert(
            "policy.matched_rules".to_string(),
            rules
                .iter()
                .map(|rule| rule.rule_id.as_str())
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    metadata
}
