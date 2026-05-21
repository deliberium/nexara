use nexara_core::{
    ActionClass, ToolDescriptor, TrustTier, validate_scope_name, validate_tool_descriptor,
    validate_tool_name,
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
    };

    validate_tool_descriptor(&descriptor).unwrap();
}
