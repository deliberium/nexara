use anyhow::Result;
use nexara_core::{
    ActionClass, CapabilitySensitivity, ToolCapability, ToolDescriptor, ToolRegistry, TrustTier,
};
use nexara_policy::{PolicyCompileOptions, PolicyCompiler, SimulationDecision, simulate_catalog};

fn main() -> Result<()> {
    let intent = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "reads orders, never refunds".to_string());
    let compiled = PolicyCompiler::default().compile(&intent, PolicyCompileOptions::default())?;
    let registry = shop_catalog();
    let simulation = simulate_catalog(&compiled.policy, &registry);

    println!("Nexara policy compiler\n");
    println!("Input:\n  {intent}\n");
    println!("Compiled:");
    for rule in &compiled.policy.rules {
        let effect = match rule.effect {
            nexara_core::PolicyEffect::Allow => "ALLOW",
            nexara_core::PolicyEffect::Deny => "DENY",
            nexara_core::PolicyEffect::RequireConfirmation => "CONFIRM",
        };
        let capabilities = rule
            .selector
            .capabilities
            .iter()
            .map(|pattern| pattern.0.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {effect:<7} {capabilities}");
    }

    println!("\nSimulation:");
    for tool in &simulation.tools {
        println!(
            "  {:<8} {}",
            match tool.decision {
                SimulationDecision::Allow => "ALLOW",
                SimulationDecision::Deny => "DENY",
                SimulationDecision::Confirm => "CONFIRM",
                SimulationDecision::MissingCapability => "MISSING",
                SimulationDecision::NoMatch => "NO MATCH",
            },
            tool.tool
        );
    }

    let denied = simulation
        .tools
        .iter()
        .find(|tool| tool.tool == "shopify.create_refund")
        .expect("refund tool exists");
    println!("\nWhy denied:\n  {}", denied.explanation);
    Ok(())
}

fn shop_catalog() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(tool(
        "shopify.get_order",
        "Fetch an order",
        capability("orders.read", &["orders"], "read", ActionClass::Read),
    ));
    registry.register(tool(
        "shopify.search_orders",
        "Search orders",
        capability("orders.search", &["orders"], "search", ActionClass::Read),
    ));
    registry.register(tool(
        "shopify.create_refund",
        "Create a refund",
        capability(
            "orders.refunds.create",
            &["orders", "refunds"],
            "create",
            ActionClass::Write,
        ),
    ));
    registry.register(tool(
        "shopify.cancel_order",
        "Cancel an order",
        capability("orders.cancel", &["orders"], "cancel", ActionClass::Write),
    ));
    registry.register(tool(
        "shopify.export_orders",
        "Export orders",
        capability("orders.export", &["orders"], "export", ActionClass::Read),
    ));
    registry
}

fn tool(name: &str, description: &str, capability: ToolCapability) -> ToolDescriptor {
    ToolDescriptor {
        name: name.to_string(),
        description: description.to_string(),
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

fn capability(
    id: &str,
    path: &[&str],
    operation: &str,
    action_class: ActionClass,
) -> ToolCapability {
    ToolCapability {
        id: id.to_string(),
        display_name: Some(id.replace('.', " ")),
        description: Some(format!("Commerce capability {id}")),
        resource: path[0].to_string(),
        resource_path: path.iter().map(|part| (*part).to_string()).collect(),
        operation: operation.to_string(),
        action_class,
        sensitivity: CapabilitySensitivity::Business,
        aliases: Vec::new(),
    }
}
