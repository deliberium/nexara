use nexara_core::ActionClass;
use nexara_core::CapabilitySensitivity;
use nexara_registry::{
    SkillConfirmationPolicy, SkillRuntime, parse_skill_manifest_str, validate_host_requirements,
};

fn valid_manifest() -> &'static str {
    r#"
name = "github-status"
version = "0.1.0"
publisher = "nexara"
description = "Read-only GitHub pull request summaries"
runtime = "remote_mcp"
trust_tier = "official"
mcp_version = "2025-12"

[host_requirements]
nexara = ">=0.1.0"
example_app = ">=0.2.0"

[source]
server_url = "https://skills.example.test/github"

[policy]
action_class = "read"
scopes = ["read"]
confirmation = "never"
allowed_hosts = ["skills.example.test"]
requires_secrets = ["github_token"]

[presentation]
display_name = "GitHub Status"
short_summary = "Read-only GitHub summaries"
categories = ["developer-tools"]
tags = ["github"]

[[capabilities]]
name = "get_pull_request_summary"
description = "Summarize a pull request"
"#
}

#[test]
fn parses_host_requirements_manifest() {
    let metadata = parse_skill_manifest_str(valid_manifest()).unwrap();
    assert_eq!(metadata.id, "nexara/github-status");
    assert_eq!(metadata.runtime, SkillRuntime::RemoteMcp);
    assert_eq!(metadata.action_class, ActionClass::Read);
    assert_eq!(metadata.confirmation, SkillConfirmationPolicy::Never);
    assert_eq!(
        metadata.host_requirements.requirement("nexara"),
        Some(">=0.1.0")
    );
}

#[test]
fn rejects_legacy_app_specific_compatibility_field() {
    let manifest = valid_manifest().replace(
        "[host_requirements]",
        "supported_host = \">=0.2.0\"\n\n[host_requirements]",
    );
    let err = parse_skill_manifest_str(&manifest).unwrap_err().to_string();
    assert!(err.contains("supported_host"));
}

#[test]
fn validates_host_requirement_versions() {
    let metadata = parse_skill_manifest_str(valid_manifest()).unwrap();
    validate_host_requirements(&metadata.host_requirements, "nexara", "0.1.0").unwrap();
    assert!(validate_host_requirements(&metadata.host_requirements, "nexara", "0.0.1").is_err());
}

#[test]
fn parses_semantic_capability_and_effect_metadata() {
    let manifest = valid_manifest().replace(
        r#"[[capabilities]]
name = "get_pull_request_summary"
description = "Summarize a pull request"
"#,
        r#"[[capabilities]]
id = "orders.refunds.create"
name = "create_order_refund"
description = "Issue a refund for an order"
display_name = "Create order refund"
resource = "orders"
resource_path = ["orders", "refunds"]
operation = "create"
action_class = "write"
sensitivity = "financial"
aliases = ["refund order", "issue refund"]

[[effects]]
resource = "refunds"
operation = "create"
writes = true
external_side_effect = true
financial_side_effect = true
irreversible = false
"#,
    );

    let metadata = parse_skill_manifest_str(&manifest).unwrap();
    let capability = metadata.capabilities.first().unwrap();
    assert_eq!(capability.id.as_deref(), Some("orders.refunds.create"));
    assert_eq!(capability.resource.as_deref(), Some("orders"));
    assert_eq!(capability.resource_path, ["orders", "refunds"]);
    assert_eq!(capability.operation.as_deref(), Some("create"));
    assert_eq!(capability.action_class, Some(ActionClass::Write));
    assert_eq!(
        capability.sensitivity,
        Some(CapabilitySensitivity::Financial)
    );
    assert_eq!(capability.aliases, ["refund order", "issue refund"]);

    let effect = metadata.effects.first().unwrap();
    assert_eq!(effect.resource, "refunds");
    assert_eq!(effect.operation, "create");
    assert!(effect.writes);
    assert!(effect.external_side_effect);
    assert!(effect.financial_side_effect);
    assert!(!effect.irreversible);
}
