use nexara_core::{PolicyDiagnosticCode, PolicyDiagnosticSeverity, PolicyEffect};
use nexara_policy::{PolicyCompileOptions, PolicyCompiler};
use std::collections::BTreeMap;

#[test]
fn compiles_orders_never_refunds() {
    let compiled = PolicyCompiler::default()
        .compile(
            "reads orders, never refunds",
            PolicyCompileOptions::default(),
        )
        .unwrap();

    assert_eq!(compiled.policy.source.text, "reads orders, never refunds");
    assert!(compiled.policy.rules.iter().any(|rule| {
        rule.effect == PolicyEffect::Allow
            && rule
                .selector
                .capabilities
                .iter()
                .any(|pattern| pattern.0 == "orders.read")
    }));
    assert!(compiled.policy.rules.iter().any(|rule| {
        rule.effect == PolicyEffect::Deny
            && rule
                .selector
                .capabilities
                .iter()
                .any(|pattern| pattern.0 == "orders.refunds.create")
    }));
}

#[test]
fn rejects_conflicting_refund_policy() {
    let error = PolicyCompiler::default()
        .compile("can refund, never refunds", PolicyCompileOptions::default())
        .unwrap_err();
    assert!(error.to_string().contains("conflicting rules"));
}

#[test]
fn warns_on_broad_manage_policy() {
    let compiled = PolicyCompiler::default()
        .compile("manage orders", PolicyCompileOptions::default())
        .unwrap();
    assert!(compiled.policy.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == PolicyDiagnosticSeverity::Warning
            && diagnostic.code == PolicyDiagnosticCode::AmbiguousVerb
    }));
}

#[test]
fn supports_host_provided_alias_dictionary() {
    let aliases = BTreeMap::from([("returns".to_string(), "refunds".to_string())]);
    let compiled = PolicyCompiler::default()
        .compile(
            "reads orders, never returns",
            PolicyCompileOptions {
                aliases,
                ..PolicyCompileOptions::default()
            },
        )
        .unwrap();

    assert!(compiled.policy.rules.iter().any(|rule| {
        rule.effect == PolicyEffect::Deny
            && rule
                .selector
                .capabilities
                .iter()
                .any(|pattern| pattern.0 == "orders.refunds.create")
    }));
}
