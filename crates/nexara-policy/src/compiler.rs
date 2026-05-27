use crate::explain::explain_contract_markdown;
use crate::grammar::parse_policy_clauses;
use crate::normalization::normalize_policy_text;
use anyhow::{Result, bail};
use nexara_core::{
    CapabilityPattern, PolicyContract, PolicyDefaults, PolicyDiagnosticSeverity, PolicyEffect,
    PolicyRule, PolicySelector, PolicySource, PolicySourceKind,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct PolicyCompileOptions {
    pub id: Option<String>,
    pub version: Option<String>,
    pub aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    pub policy: PolicyContract,
}

impl CompiledPolicy {
    pub fn explain_markdown(&self) -> String {
        explain_contract_markdown(&self.policy)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PolicyCompiler {
    _private: (),
}

impl PolicyCompiler {
    pub fn compile(&self, input: &str, options: PolicyCompileOptions) -> Result<CompiledPolicy> {
        let normalized = apply_aliases(normalize_policy_text(input), &options.aliases);
        let (parsed, diagnostics) = parse_policy_clauses(&normalized);
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == PolicyDiagnosticSeverity::Error)
        {
            bail!(
                "policy contains errors: {}",
                diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.severity == PolicyDiagnosticSeverity::Error)
                    .map(|diagnostic| diagnostic.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        let mut rules = Vec::new();
        for (idx, clause) in parsed.clauses.iter().enumerate() {
            let capability = clause.capability();
            let selector =
                if matches!(clause.effect, PolicyEffect::Deny) && clause.resource == "refunds" {
                    PolicySelector {
                        capabilities: vec![
                            CapabilityPattern("refunds.create".to_string()),
                            CapabilityPattern("orders.refunds.create".to_string()),
                        ],
                        ..PolicySelector::default()
                    }
                } else {
                    let mut capabilities = vec![CapabilityPattern(capability)];
                    if clause.operation == "read" {
                        capabilities.push(CapabilityPattern(format!("{}.search", clause.resource)));
                    }
                    PolicySelector {
                        capabilities,
                        ..PolicySelector::default()
                    }
                };
            rules.push(PolicyRule {
                id: format!(
                    "{}-{}-{}",
                    match clause.effect {
                        PolicyEffect::Allow => "allow",
                        PolicyEffect::Deny => "deny",
                        PolicyEffect::RequireConfirmation => "confirm",
                    },
                    clause.resource,
                    clause.operation
                ),
                effect: clause.effect,
                selector,
                condition: None,
                reason: clause.reason.clone(),
            });
            if rules[idx].effect == PolicyEffect::Allow && clause.operation == "read" {
                rules.push(PolicyRule {
                    id: format!("confirm-{}-export", clause.resource),
                    effect: PolicyEffect::RequireConfirmation,
                    selector: PolicySelector {
                        capabilities: vec![CapabilityPattern(format!(
                            "{}.export",
                            clause.resource
                        ))],
                        ..PolicySelector::default()
                    },
                    condition: None,
                    reason: format!("exporting {} should be reviewed", clause.resource),
                });
            }
        }

        Ok(CompiledPolicy {
            policy: PolicyContract {
                id: options.id.unwrap_or_else(|| stable_policy_id(&normalized)),
                version: options.version.unwrap_or_else(|| "0.1.0".to_string()),
                source: PolicySource {
                    kind: PolicySourceKind::OneLine,
                    text: input.to_string(),
                },
                rules,
                defaults: PolicyDefaults::default(),
                diagnostics,
                created_at: None,
            },
        })
    }
}

fn stable_policy_id(input: &str) -> String {
    let slug = input
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    format!("policy-{slug}")
}

fn apply_aliases(input: String, aliases: &BTreeMap<String, String>) -> String {
    aliases.iter().fold(input, |current, (from, to)| {
        current
            .split_whitespace()
            .map(|word| {
                if word == normalize_policy_text(from) {
                    normalize_policy_text(to)
                } else {
                    word.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    })
}
