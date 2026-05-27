use nexara_core::{
    ActionClass, CapabilityPattern, CapabilitySensitivity, PolicyContract, PolicyDefaults,
    PolicyEffect, PolicyEvaluationDecision, PolicyRule, PolicySelector, PolicySource,
    PolicySourceKind, ToolCapability, ToolDescriptor, TrustTier,
};

fn policy() -> PolicyContract {
    PolicyContract {
        id: "orders-policy".to_string(),
        version: "0.1.0".to_string(),
        source: PolicySource {
            kind: PolicySourceKind::OneLine,
            text: "reads orders, never refunds".to_string(),
        },
        rules: vec![
            PolicyRule {
                id: "allow-orders-read".to_string(),
                effect: PolicyEffect::Allow,
                selector: PolicySelector {
                    capabilities: vec![CapabilityPattern("orders.read".to_string())],
                    ..PolicySelector::default()
                },
                condition: None,
                reason: "reads orders".to_string(),
            },
            PolicyRule {
                id: "deny-orders-refunds-create".to_string(),
                effect: PolicyEffect::Deny,
                selector: PolicySelector {
                    capabilities: vec![CapabilityPattern("orders.refunds.create".to_string())],
                    ..PolicySelector::default()
                },
                condition: None,
                reason: "never refunds".to_string(),
            },
            PolicyRule {
                id: "confirm-orders-export".to_string(),
                effect: PolicyEffect::RequireConfirmation,
                selector: PolicySelector {
                    capabilities: vec![CapabilityPattern("orders.export".to_string())],
                    ..PolicySelector::default()
                },
                condition: None,
                reason: "exporting orders needs review".to_string(),
            },
        ],
        defaults: PolicyDefaults::default(),
        diagnostics: Vec::new(),
        created_at: None,
    }
}

fn tool(name: &str, capability_id: &str, operation: &str) -> ToolDescriptor {
    ToolDescriptor {
        name: name.to_string(),
        description: format!("{name} tool"),
        tags: Vec::new(),
        scopes: vec!["read".to_string()],
        trust_tier: TrustTier::RemoteNexara,
        action_class: ActionClass::Read,
        enabled: true,
        input_schema: None,
        guidance: None,
        capabilities: vec![ToolCapability {
            id: capability_id.to_string(),
            display_name: Some(capability_id.to_string()),
            description: Some(format!("Capability {capability_id}")),
            resource: "orders".to_string(),
            resource_path: capability_id
                .split('.')
                .take_while(|segment| *segment != operation)
                .map(ToString::to_string)
                .collect(),
            operation: operation.to_string(),
            action_class: ActionClass::Read,
            sensitivity: CapabilitySensitivity::Business,
            aliases: Vec::new(),
        }],
        effects: Vec::new(),
    }
}

#[test]
fn policy_contract_serializes_stably() {
    let json = serde_json::to_string_pretty(&policy()).unwrap();

    assert!(json.contains("\"id\": \"orders-policy\""));
    assert!(json.contains("\"effect\": \"allow\""));
    assert!(json.contains("\"effect\": \"deny\""));
    assert!(json.contains("\"deny_missing_capabilities\": true"));

    let roundtrip: PolicyContract = serde_json::from_str(&json).unwrap();
    assert_eq!(roundtrip, policy());
}

#[test]
fn rule_matching_and_precedence_are_enforced() {
    let contract = policy();

    let allowed = contract.evaluate_tool(&tool("get_order", "orders.read", "read"), false);
    assert_eq!(allowed.decision, PolicyEvaluationDecision::Allowed);
    assert_eq!(allowed.matched_rules[0].rule_id, "allow-orders-read");

    let denied = contract.evaluate_tool(
        &tool("create_refund", "orders.refunds.create", "create"),
        true,
    );
    assert_eq!(denied.decision, PolicyEvaluationDecision::Denied);
    assert!(denied.explanation.contains("never refunds"));

    let confirm = contract.evaluate_tool(&tool("export_orders", "orders.export", "export"), false);
    assert_eq!(
        confirm.decision,
        PolicyEvaluationDecision::ConfirmationRequired
    );

    let confirmed = contract.evaluate_tool(&tool("export_orders", "orders.export", "export"), true);
    assert_eq!(
        confirmed.decision,
        PolicyEvaluationDecision::NoMatchingAllow
    );
}

#[test]
fn missing_capability_metadata_denies_when_configured() {
    let mut unlabelled = ToolDescriptor::read_only(
        "legacy.search_orders",
        "Legacy order search without semantic metadata",
    );
    unlabelled.trust_tier = TrustTier::RemoteNexara;

    let evaluation = policy().evaluate_tool(&unlabelled, false);
    assert_eq!(
        evaluation.decision,
        PolicyEvaluationDecision::DescriptorMissingCapabilities
    );
    assert!(evaluation.explanation.contains("semantic capability"));
}
