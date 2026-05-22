#[test]
fn facade_reexports_core_types_by_default() {
    let descriptor = nexara::ToolDescriptor::read_only("search", "Search docs");

    assert_eq!(descriptor.name, "search");
    assert_eq!(nexara::TrustProfile::Observe.as_str(), "observe");
}

#[test]
#[cfg(feature = "runtime")]
fn facade_exposes_runtime_feature_module() {
    let config = nexara::runtime::NexaraRuntimeConfig::default();

    assert_eq!(config.max_tools_per_request, 8);
}

#[test]
#[cfg(feature = "registry")]
fn facade_exposes_registry_feature_module() {
    let requirements = nexara::registry::HostRequirements::default();

    assert_eq!(requirements.requirement("nexara"), None);
}

#[test]
#[cfg(feature = "secrets")]
fn facade_exposes_secrets_feature_module() {
    let secret = nexara::secrets::SecretValue::new("hidden");

    assert_eq!(secret.expose(), "hidden");
}

#[test]
#[cfg(feature = "remote")]
fn facade_exposes_remote_feature_module() {
    let endpoint = nexara::remote::RemoteEndpoint {
        base_url: "https://tools.example.com".to_string(),
    };

    assert_eq!(endpoint.base_url, "https://tools.example.com");
}

#[test]
#[cfg(feature = "learning")]
fn facade_exposes_learning_feature_module() {
    let scorer = nexara::learning::LearningScorer::default();

    assert_eq!(scorer.score_events(&[]), 0.0);
}

#[test]
#[cfg(feature = "compat-qwen")]
fn facade_exposes_qwen_compat_feature_module() {
    let profile = nexara::compat_qwen::build_qwen_compatibility_profile("qwen2.5-coder");

    assert!(profile.is_qwen_family);
}

#[test]
#[cfg(feature = "server")]
fn facade_exposes_server_feature_module() {
    let auth = nexara::server::ServerAuth::require("token");

    assert!(format!("{auth:?}").contains("[redacted]"));
}
