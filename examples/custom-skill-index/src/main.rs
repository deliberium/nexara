use nexara_registry::installed::InstalledSkillStore;
use nexara_registry::{
    InstalledSkillRecord, InstalledSkillState, JsonFileInstalledSkillStore,
    parse_skill_manifest_str,
};
use sha2::{Digest, Sha256};

const MANIFEST: &str = r#"
name = "custom-search"
version = "0.1.0"
publisher = "example"
description = "Searches an example corpus"
runtime = "nexara_http"
trust_tier = "community"
mcp_version = "2025-12"

[host_requirements]
nexara = ">=0.1.0"
example_app = ">=0.2.0"

[source]
server_url = "https://tools.example.com"

[policy]
action_class = "read"
scopes = ["read"]
confirmation = "never"
allowed_hosts = ["tools.example.com"]
requires_secrets = ["EXAMPLE_API_KEY"]

[presentation]
display_name = "Custom Search"
short_summary = "Search an example corpus"
categories = ["search"]
tags = ["example", "search"]

[[capabilities]]
name = "search"
description = "Search the example corpus"
parameters = { query = "string" }
"#;

fn main() -> anyhow::Result<()> {
    let metadata = parse_skill_manifest_str(MANIFEST)?;
    let digest = Sha256::digest(MANIFEST.as_bytes());
    println!("manifest id: {}", metadata.id);
    println!("manifest sha256: {digest:x}");

    let tempdir = tempfile::tempdir()?;
    let store = JsonFileInstalledSkillStore::new(tempdir.path().join("installed-skills.json"));
    store.put(InstalledSkillRecord {
        metadata,
        state: InstalledSkillState {
            installed: true,
            enabled: false,
        },
    })?;
    println!("{}", serde_json::to_string_pretty(&store.list()?)?);
    Ok(())
}
