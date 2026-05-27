use nexara_core::{
    ActionClass, CapabilitySensitivity, ToolCapability, ToolDescriptor, TrustTier,
    validate_scope_name, validate_tool_descriptor, validate_tool_name,
};

#[test]
fn validation_rejects_empty_tool_names() {
    assert!(validate_tool_name("").is_err());
}

#[test]
fn validation_rejects_slashes_in_tool_names() {
    assert!(validate_tool_name("bad/tool").is_err());
}

#[test]
fn validation_accepts_scoped_scope_names() {
    assert!(validate_scope_name("calendar:read").is_ok());
}

#[test]
fn validation_checks_full_descriptor() {
    let descriptor = ToolDescriptor {
        name: "calendar.list".to_string(),
        description: "List events".to_string(),
        tags: vec![],
        scopes: vec!["calendar:read".to_string()],
        trust_tier: TrustTier::Builtin,
        action_class: ActionClass::Read,
        enabled: true,
        input_schema: None,
        guidance: None,
        capabilities: vec![ToolCapability {
            id: "calendar.events.read".to_string(),
            display_name: Some("Read calendar events".to_string()),
            description: Some("List calendar events without modifying them".to_string()),
            resource: "calendar".to_string(),
            resource_path: vec!["calendar".to_string(), "events".to_string()],
            operation: "read".to_string(),
            action_class: ActionClass::Read,
            sensitivity: CapabilitySensitivity::Business,
            aliases: vec!["list events".to_string()],
        }],
        effects: Vec::new(),
    };

    validate_tool_descriptor(&descriptor).unwrap();
}

#[test]
fn validation_rejects_deeper_capability_without_labels() {
    let mut descriptor = ToolDescriptor::read_only("calendar.list", "List events");
    descriptor.capabilities.push(ToolCapability {
        id: "calendar.events.read".to_string(),
        display_name: None,
        description: None,
        resource: "calendar".to_string(),
        resource_path: vec!["calendar".to_string(), "events".to_string()],
        operation: "read".to_string(),
        action_class: ActionClass::Read,
        sensitivity: CapabilitySensitivity::Business,
        aliases: Vec::new(),
    });

    assert!(validate_tool_descriptor(&descriptor).is_err());
}
