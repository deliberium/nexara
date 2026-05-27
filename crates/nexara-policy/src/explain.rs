use nexara_core::{PolicyContract, PolicyEffect};

pub fn explain_contract_markdown(contract: &PolicyContract) -> String {
    let mut output = String::new();
    output.push_str("# Nexara Policy Contract\n\n");
    output.push_str(&format!("Input: `{}`\n\n", contract.source.text));
    output.push_str("## Rules\n\n");
    for rule in &contract.rules {
        let effect = match rule.effect {
            PolicyEffect::Allow => "ALLOW",
            PolicyEffect::Deny => "DENY",
            PolicyEffect::RequireConfirmation => "CONFIRM",
        };
        let capabilities = rule
            .selector
            .capabilities
            .iter()
            .map(|pattern| pattern.0.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!("- {effect} {capabilities} — {}\n", rule.reason));
    }
    if !contract.diagnostics.is_empty() {
        output.push_str("\n## Diagnostics\n\n");
        for diagnostic in &contract.diagnostics {
            output.push_str(&format!(
                "- {:?}: {}\n",
                diagnostic.severity, diagnostic.message
            ));
        }
    }
    output
}
