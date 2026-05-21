use nexara_core::{
    ActionClass, ToolBroker, ToolDescriptor, ToolRegistry, ToolSelectionRequest, TrustTier,
};

fn descriptor(name: &str, tags: &[&str]) -> ToolDescriptor {
    ToolDescriptor {
        name: name.to_string(),
        description: "test tool".to_string(),
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        scopes: vec!["read".to_string()],
        trust_tier: TrustTier::Builtin,
        action_class: ActionClass::Read,
        enabled: true,
        input_schema: None,
        guidance: None,
    }
}

#[test]
fn broker_prefers_matching_tags() {
    let mut registry = ToolRegistry::new();
    registry.register(descriptor("memory_recall", &["memory", "recall"]));
    registry.register(descriptor("get_voice_status", &["voice", "status"]));
    let broker = ToolBroker::new();

    let selected = broker.select(
        &registry,
        &ToolSelectionRequest {
            prompt: "check voice status".to_string(),
            scopes: vec!["read".to_string()],
            max_tools: 1,
        },
    );

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].name, "get_voice_status");
}

#[test]
fn broker_filters_by_scope() {
    let mut registry = ToolRegistry::new();
    let mut tool = descriptor("memory_recall", &["memory"]);
    tool.scopes = vec!["write".to_string()];
    registry.register(tool);
    let broker = ToolBroker::new();

    let selected = broker.select(
        &registry,
        &ToolSelectionRequest {
            prompt: "memory".to_string(),
            scopes: vec!["read".to_string()],
            max_tools: 4,
        },
    );

    assert!(selected.is_empty());
}

#[test]
fn broker_can_select_remote_read_tools() {
    let mut registry = ToolRegistry::new();
    registry.register(descriptor("get_voice_status", &["voice", "status"]));
    let mut remote = descriptor("remote_search", &["web", "search"]);
    remote.trust_tier = TrustTier::RemoteMcp;
    registry.register(remote);
    let broker = ToolBroker::new();

    let selected = broker.select(
        &registry,
        &ToolSelectionRequest {
            prompt: "search the web for release notes".to_string(),
            scopes: vec!["read".to_string()],
            max_tools: 1,
        },
    );

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].name, "remote_search");
    assert_eq!(selected[0].trust_tier, TrustTier::RemoteMcp);
}
