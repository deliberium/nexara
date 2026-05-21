use nexara_compat_qwen::{build_qwen_compatibility_profile, import_qwen_mcp_servers};

#[test]
fn detects_qwen_family_models() {
    let profile = build_qwen_compatibility_profile("Qwen2.5-Coder");
    assert!(profile.is_qwen_family);
    assert!(profile.tool_prompt_guidance.is_some());
}

#[test]
fn imports_http_mcp_servers_as_unapproved_endpoints() {
    let records = import_qwen_mcp_servers(
        r#"{
          "mcpServers": {
            "github": { "url": "https://tools.example.test" }
          }
        }"#,
    )
    .unwrap();
    assert_eq!(records.len(), 1);
    assert!(!records[0].approved);
}

#[test]
fn rejects_command_transport() {
    let err = import_qwen_mcp_servers(
        r#"{
          "mcpServers": {
            "local": { "command": "node" }
          }
        }"#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("command/process"));
}
