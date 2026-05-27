use nexara_core::{
    ActionClass, CapabilitySensitivity, ToolCapability, ToolDescriptor, ToolRegistry, TrustTier,
};
use nexara_policy::{PolicyCompileOptions, PolicyCompiler, SimulationDecision, simulate_catalog};

fn tool(name: &str, capability: ToolCapability) -> ToolDescriptor {
    ToolDescriptor {
        name: name.to_string(),
        description: name.to_string(),
        tags: Vec::new(),
        scopes: vec!["read".to_string()],
        trust_tier: TrustTier::RemoteNexara,
        action_class: capability.action_class,
        enabled: true,
        input_schema: None,
        guidance: None,
        capabilities: vec![capability],
        effects: Vec::new(),
    }
}

fn capability(id: &str, path: &[&str], op: &str, action: ActionClass) -> ToolCapability {
    ToolCapability {
        id: id.to_string(),
        display_name: Some(id.to_string()),
        description: Some(format!("Capability {id}")),
        resource: path[0].to_string(),
        resource_path: path.iter().map(|part| (*part).to_string()).collect(),
        operation: op.to_string(),
        action_class: action,
        sensitivity: CapabilitySensitivity::Business,
        aliases: Vec::new(),
    }
}

#[test]
fn simulates_policy_against_shop_catalog() {
    let compiled = PolicyCompiler::default()
        .compile(
            "reads orders, never refunds",
            PolicyCompileOptions::default(),
        )
        .unwrap();
    let mut registry = ToolRegistry::new();
    registry.register(tool(
        "shopify.get_order",
        capability("orders.read", &["orders"], "read", ActionClass::Read),
    ));
    registry.register(tool(
        "shopify.create_refund",
        capability(
            "orders.refunds.create",
            &["orders", "refunds"],
            "create",
            ActionClass::Write,
        ),
    ));
    registry.register(ToolDescriptor::read_only(
        "shopify.legacy_lookup",
        "Legacy lookup without semantic metadata",
    ));

    let simulation = simulate_catalog(&compiled.policy, &registry);
    let decision = |name: &str| {
        simulation
            .tools
            .iter()
            .find(|tool| tool.tool == name)
            .unwrap()
            .decision
    };

    assert_eq!(decision("shopify.get_order"), SimulationDecision::Allow);
    assert_eq!(decision("shopify.create_refund"), SimulationDecision::Deny);
    assert_eq!(
        decision("shopify.legacy_lookup"),
        SimulationDecision::MissingCapability
    );
}
