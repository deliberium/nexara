use nexara_core::{PolicyContract, PolicyEvaluationDecision, ToolDescriptor, ToolRegistry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogSimulation {
    pub policy_id: String,
    pub tools: Vec<SimulatedToolDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimulatedToolDecision {
    pub tool: String,
    pub decision: SimulationDecision,
    pub explanation: String,
    pub matched_rules: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SimulationDecision {
    Allow,
    Deny,
    Confirm,
    MissingCapability,
    NoMatch,
}

pub fn simulate_catalog(policy: &PolicyContract, registry: &ToolRegistry) -> CatalogSimulation {
    let mut tools = registry
        .all()
        .iter()
        .map(|tool| simulate_tool(policy, tool))
        .collect::<Vec<_>>();
    tools.sort_by(|left, right| left.tool.cmp(&right.tool));
    CatalogSimulation {
        policy_id: policy.id.clone(),
        tools,
    }
}

fn simulate_tool(policy: &PolicyContract, tool: &ToolDescriptor) -> SimulatedToolDecision {
    let evaluation = policy.evaluate_tool(tool, false);
    let decision = match evaluation.decision {
        PolicyEvaluationDecision::Allowed => SimulationDecision::Allow,
        PolicyEvaluationDecision::Denied => SimulationDecision::Deny,
        PolicyEvaluationDecision::ConfirmationRequired => SimulationDecision::Confirm,
        PolicyEvaluationDecision::NoMatchingAllow => SimulationDecision::NoMatch,
        PolicyEvaluationDecision::DescriptorMissingCapabilities => {
            SimulationDecision::MissingCapability
        }
    };
    SimulatedToolDecision {
        tool: tool.name.clone(),
        decision,
        explanation: evaluation.explanation,
        matched_rules: evaluation
            .matched_rules
            .into_iter()
            .map(|rule| rule.rule_id)
            .collect(),
    }
}
