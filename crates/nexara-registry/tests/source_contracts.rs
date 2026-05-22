use std::collections::BTreeMap;

use nexara_core::ActionClass;
use nexara_registry::{
    Ed25519SignatureVerifier, FileSkillIndexSource, HostRequirements, SignatureVerifier,
    SignedSkillIndex, SignedSkillIndexEntry, SignedSkillIndexPublisher, SignedSkillIndexSignature,
    SkillIndexSource, SkillPublisherTrust, SkillRuntime,
};
use sha2::{Digest, Sha256};

#[tokio::test]
async fn file_index_source_fetches_index_and_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.toml");
    let manifest = "name = \"fixture\"\n";
    std::fs::write(&manifest_path, manifest).unwrap();

    let mut index = index_fixture();
    index.skills[0].manifest_url = manifest_path.to_string_lossy().to_string();
    let index_path = dir.path().join("index.json");
    std::fs::write(&index_path, serde_json::to_vec(&index).unwrap()).unwrap();

    let source = FileSkillIndexSource::new(index_path);
    let fetched = source.fetch_index().await.unwrap();
    let fetched_manifest = source.fetch_manifest(&fetched.skills[0]).await.unwrap();

    assert_eq!(fetched.publisher.id, "example");
    assert_eq!(fetched_manifest, manifest);
}

#[test]
fn manifest_sha_verification_accepts_match_and_rejects_mismatch() {
    let verifier = Ed25519SignatureVerifier::new(BTreeMap::new());
    let bytes = b"manifest bytes";
    let expected = Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        });

    verifier.verify_manifest_bytes(bytes, &expected).unwrap();
    let err = verifier
        .verify_manifest_bytes(bytes, "000000")
        .unwrap_err()
        .to_string();
    assert!(err.contains("manifest sha mismatch"));
}

fn index_fixture() -> SignedSkillIndex {
    let mut requirements = BTreeMap::new();
    requirements.insert("nexara".to_string(), ">=0.1.0".to_string());

    SignedSkillIndex {
        schema_version: "2026-05".to_string(),
        generated_at: "2026-05-22T00:00:00Z".to_string(),
        publisher: SignedSkillIndexPublisher {
            id: "example".to_string(),
            display_name: "Example".to_string(),
            public_key_id: "fixture-key".to_string(),
        },
        signature: SignedSkillIndexSignature {
            algorithm: "ed25519".to_string(),
            key_id: "fixture-key".to_string(),
            signature: "fixture".to_string(),
        },
        skills: vec![SignedSkillIndexEntry {
            id: "example/source-fixture".to_string(),
            version: "0.1.0".to_string(),
            manifest_url: "manifest.toml".to_string(),
            manifest_sha256: "unused".to_string(),
            trust_tier: SkillPublisherTrust::Community,
            runtime: SkillRuntime::NexaraHttp,
            action_class: ActionClass::Read,
            scopes: vec!["read".to_string()],
            categories: Vec::new(),
            tags: Vec::new(),
            requires_secrets: Vec::new(),
            requires_confirmation: false,
            host_requirements: HostRequirements { requirements },
            mcp_version: "2025-12".to_string(),
            summary: "Fixture".to_string(),
        }],
    }
}
