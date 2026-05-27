use crate::ast::{ParsedPolicy, PolicyClause};
use crate::diagnostics::{error, warning};
use crate::normalization::{
    infer_resource_from_operation, normalize_operation, normalize_resource,
};
use nexara_core::{PolicyDiagnostic, PolicyDiagnosticCode, PolicyDiagnosticSeverity, PolicyEffect};

pub fn parse_policy_clauses(input: &str) -> (ParsedPolicy, Vec<PolicyDiagnostic>) {
    let mut clauses = Vec::new();
    let mut diagnostics = Vec::new();

    for raw_clause in input
        .split([',', '.'])
        .map(str::trim)
        .filter(|c| !c.is_empty())
    {
        if raw_clause.contains("anything") {
            diagnostics.push(error(
                PolicyDiagnosticCode::OverBroadAllow,
                "broad write policy requires explicit operations",
            ));
            continue;
        }
        if raw_clause.contains("manage") {
            diagnostics.push(warning(
                PolicyDiagnosticCode::AmbiguousVerb,
                "\"manage\" is broad; compile result should be reviewed",
            ));
        }
        if let Some(clause) = parse_clause(raw_clause) {
            clauses.push(clause);
        } else {
            diagnostics.push(warning(
                PolicyDiagnosticCode::UnknownOperation,
                format!("could not parse policy clause '{raw_clause}'"),
            ));
        }
    }

    detect_conflicts(&clauses, &mut diagnostics);
    (ParsedPolicy { clauses }, diagnostics)
}

fn parse_clause(raw: &str) -> Option<PolicyClause> {
    let words = raw.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() {
        return None;
    }

    let deny = raw.starts_with("never ")
        || raw.starts_with("cannot ")
        || raw.starts_with("can't ")
        || raw.starts_with("deny ")
        || raw.contains(" cannot ")
        || raw.contains(" never ");
    let confirm = raw.contains("confirm before") || raw.starts_with("must confirm");
    let read_only = raw.contains("read-only") || raw.contains("read only");

    if read_only {
        let resource = words.last().map(|word| normalize_resource(word))?;
        return Some(PolicyClause {
            effect: PolicyEffect::Allow,
            resource,
            operation: "read".to_string(),
            reason: raw.to_string(),
        });
    }

    let effect = if confirm {
        PolicyEffect::RequireConfirmation
    } else if deny {
        PolicyEffect::Deny
    } else {
        PolicyEffect::Allow
    };

    let meaningful = words
        .iter()
        .copied()
        .filter(|word| {
            !matches!(
                *word,
                "can" | "may" | "must" | "before" | "allow" | "deny" | "never" | "cannot" | "to"
            )
        })
        .collect::<Vec<_>>();
    if meaningful.is_empty() {
        return None;
    }

    let operation_word = meaningful[0];
    let operation = normalize_operation(operation_word).unwrap_or_else(|| {
        if deny {
            "create".to_string()
        } else {
            "read".to_string()
        }
    });
    let resource = meaningful
        .get(1)
        .map(|word| normalize_resource(word))
        .or_else(|| infer_resource_from_operation(operation_word))
        .unwrap_or_else(|| normalize_resource(operation_word));

    Some(PolicyClause {
        effect,
        resource,
        operation,
        reason: raw.to_string(),
    })
}

fn detect_conflicts(clauses: &[PolicyClause], diagnostics: &mut Vec<PolicyDiagnostic>) {
    for (idx, left) in clauses.iter().enumerate() {
        for right in clauses.iter().skip(idx + 1) {
            if left.capability() == right.capability() && left.effect != right.effect {
                diagnostics.push(PolicyDiagnostic {
                    severity: PolicyDiagnosticSeverity::Error,
                    code: PolicyDiagnosticCode::ConflictingRule,
                    message: format!("conflicting rules for {}", left.capability()),
                });
            }
        }
    }
}
