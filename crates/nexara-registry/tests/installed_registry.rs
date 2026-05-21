use nexara_registry::{
    InstalledSkillRecord, InstalledSkillState, JsonFileInstalledSkillStore,
    installed::InstalledSkillStore, parse_skill_manifest_str,
};

#[test]
fn json_store_roundtrips_records() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonFileInstalledSkillStore::new(dir.path().join("installed.json"));
    let metadata = parse_skill_manifest_str(
        r#"
name = "status"
version = "0.1.0"
publisher = "nexara"
description = "Status"
runtime = "builtin_bundle"
trust_tier = "official"
mcp_version = "2025-12"

[host_requirements]
nexara = ">=0.1.0"

[source]
builtin_bundle_id = "status"

[policy]
action_class = "read"
scopes = ["read"]
confirmation = "never"
"#,
    )
    .unwrap();

    store
        .put(InstalledSkillRecord {
            metadata: metadata.clone(),
            state: InstalledSkillState {
                installed: true,
                enabled: true,
            },
        })
        .unwrap();

    assert_eq!(store.list().unwrap().len(), 1);
    assert!(store.get(&metadata.id).unwrap().is_some());
    store.remove(&metadata.id).unwrap();
    assert!(store.get(&metadata.id).unwrap().is_none());
}
