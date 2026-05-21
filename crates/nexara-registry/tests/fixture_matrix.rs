use std::collections::BTreeMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::{Signer, SigningKey};
use nexara_core::ActionClass;
use nexara_registry::{
    Ed25519SignatureVerifier, HostRequirements, SignatureVerifier, SignedSkillIndex,
    SignedSkillIndexEntry, SignedSkillIndexPublisher, SignedSkillIndexSignature,
    SkillPublisherTrust, SkillRuntime, parse_skill_manifest_str, validate_host_requirements,
};

const VALID: &str = r#"
name = "fixture-search"
version = "0.1.0"
publisher = "example"
description = "Fixture skill"
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
requires_secrets = []
"#;

#[test]
fn valid_fixture_parses() {
    let manifest = parse_skill_manifest_str(VALID).unwrap();
    assert_eq!(manifest.id, "example/fixture-search");
}

#[test]
fn unsupported_host_requirement_fails_validation() {
    let manifest =
        parse_skill_manifest_str(&VALID.replace("nexara = \">=0.1.0\"", "nexara = \">=999.0.0\""))
            .unwrap();
    let err = validate_host_requirements(&manifest.host_requirements, "nexara", "0.1.0")
        .unwrap_err()
        .to_string();
    assert!(err.contains("does not satisfy"));
}

#[test]
fn missing_secret_fixture_surfaces_required_secret() {
    let manifest = parse_skill_manifest_str(&VALID.replace(
        "requires_secrets = []",
        "requires_secrets = [\"API_TOKEN\"]",
    ))
    .unwrap();
    assert_eq!(manifest.requires_secrets, vec!["API_TOKEN"]);
}

#[test]
fn confirmation_required_fixture_surfaces_policy() {
    let manifest = parse_skill_manifest_str(
        &VALID.replace("confirmation = \"never\"", "confirmation = \"on_write\""),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&manifest.confirmation).unwrap(),
        serde_json::json!("on_write")
    );
}

#[test]
fn host_requirements_can_hold_multiple_hosts() {
    let requirements: HostRequirements = toml::from_str(
        r#"
nexara = ">=0.1.0"
custom_host = "^1.2"
"#,
    )
    .unwrap();
    assert_eq!(requirements.requirement("custom_host"), Some("^1.2"));
}

#[test]
fn signed_index_fixture_verifies() {
    let (index, verifier) = signed_index_fixture(skill_entry(
        "example/signed-search",
        ">=0.1.0",
        Vec::new(),
        false,
    ));

    verifier.verify_index(&index).unwrap();
}

#[test]
fn tampered_signed_index_fixture_fails_verification() {
    let (mut index, verifier) = signed_index_fixture(skill_entry(
        "example/signed-search",
        ">=0.1.0",
        Vec::new(),
        false,
    ));
    index.skills[0].summary = "Tampered summary after signing".to_string();

    let err = verifier.verify_index(&index).unwrap_err().to_string();
    assert!(!err.is_empty());
}

#[test]
fn unsupported_signed_index_fixture_fails_host_validation() {
    let (index, verifier) = signed_index_fixture(skill_entry(
        "example/unsupported-host",
        ">=999.0.0",
        Vec::new(),
        false,
    ));
    verifier.verify_index(&index).unwrap();

    let err = validate_host_requirements(&index.skills[0].host_requirements, "nexara", "0.1.0")
        .unwrap_err()
        .to_string();
    assert!(err.contains("does not satisfy"));
}

#[test]
fn missing_secret_signed_index_fixture_surfaces_required_secret() {
    let (index, verifier) = signed_index_fixture(skill_entry(
        "example/missing-secret",
        ">=0.1.0",
        vec!["API_TOKEN".to_string()],
        false,
    ));
    verifier.verify_index(&index).unwrap();

    assert_eq!(index.skills[0].requires_secrets, vec!["API_TOKEN"]);
}

#[test]
fn confirmation_required_signed_index_fixture_surfaces_policy() {
    let (index, verifier) = signed_index_fixture(skill_entry(
        "example/confirmation-required",
        ">=0.1.0",
        Vec::new(),
        true,
    ));
    verifier.verify_index(&index).unwrap();

    assert!(index.skills[0].requires_confirmation);
}

fn signed_index_fixture(
    entry: SignedSkillIndexEntry,
) -> (SignedSkillIndex, Ed25519SignatureVerifier) {
    let signing_key = SigningKey::from_bytes(&[7; 32]);
    let verifying_key = signing_key.verifying_key();
    let mut trusted_keys = BTreeMap::new();
    trusted_keys.insert("fixture-key".to_string(), verifying_key);

    let mut index = SignedSkillIndex {
        schema_version: "2026-05".to_string(),
        generated_at: "2026-05-22T00:00:00Z".to_string(),
        publisher: SignedSkillIndexPublisher {
            id: "example".to_string(),
            display_name: "Example Publisher".to_string(),
            public_key_id: "fixture-key".to_string(),
        },
        signature: SignedSkillIndexSignature {
            algorithm: "ed25519".to_string(),
            key_id: "fixture-key".to_string(),
            signature: String::new(),
        },
        skills: vec![entry],
    };

    let payload = canonical_signed_index_payload(&index);
    let signature = signing_key.sign(&payload);
    index.signature.signature = BASE64.encode(signature.to_bytes());

    (index, Ed25519SignatureVerifier::new(trusted_keys))
}

fn canonical_signed_index_payload(index: &SignedSkillIndex) -> Vec<u8> {
    let mut clone = index.clone();
    clone.signature.signature.clear();
    serde_json::to_vec(&clone).unwrap()
}

fn skill_entry(
    id: &str,
    nexara_requirement: &str,
    requires_secrets: Vec<String>,
    requires_confirmation: bool,
) -> SignedSkillIndexEntry {
    let mut requirements = BTreeMap::new();
    requirements.insert("nexara".to_string(), nexara_requirement.to_string());

    SignedSkillIndexEntry {
        id: id.to_string(),
        version: "0.1.0".to_string(),
        manifest_url: format!("https://registry.example.com/{id}/manifest.toml"),
        manifest_sha256: "7f83b1657ff1fc53b92dc18148a1d65dfa135d0f114b6b26f1b4d3e6a1d8f1d6"
            .to_string(),
        trust_tier: SkillPublisherTrust::Community,
        runtime: SkillRuntime::NexaraHttp,
        action_class: if requires_confirmation {
            ActionClass::Write
        } else {
            ActionClass::Read
        },
        scopes: vec!["read".to_string()],
        categories: vec!["fixtures".to_string()],
        tags: vec!["signed-index".to_string()],
        requires_secrets,
        requires_confirmation,
        host_requirements: HostRequirements { requirements },
        mcp_version: "2025-12".to_string(),
        summary: "Signed fixture skill".to_string(),
    }
}
