use crate::contract::SignatureVerifier;
use crate::manifest::SignedSkillIndex;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Ed25519SignatureVerifier {
    trusted_keys: BTreeMap<String, VerifyingKey>,
}

impl Ed25519SignatureVerifier {
    pub fn new(trusted_keys: BTreeMap<String, VerifyingKey>) -> Self {
        Self { trusted_keys }
    }

    fn canonical_index_payload(index: &SignedSkillIndex) -> anyhow::Result<Vec<u8>> {
        let mut clone = index.clone();
        clone.signature.signature.clear();
        Ok(serde_json::to_vec(&clone)?)
    }
}

impl SignatureVerifier for Ed25519SignatureVerifier {
    fn verify_index(&self, index: &SignedSkillIndex) -> anyhow::Result<()> {
        if index.signature.algorithm != "ed25519" {
            anyhow::bail!("unsupported signature algorithm");
        }
        let key = self
            .trusted_keys
            .get(&index.signature.key_id)
            .ok_or_else(|| anyhow::anyhow!("unknown key id '{}'", index.signature.key_id))?;
        let signature_bytes = BASE64.decode(&index.signature.signature)?;
        let signature = Signature::from_slice(&signature_bytes)?;
        let payload = Self::canonical_index_payload(index)?;
        key.verify(&payload, &signature)?;
        Ok(())
    }

    fn verify_manifest_bytes(&self, bytes: &[u8], expected_sha256: &str) -> anyhow::Result<()> {
        let digest = Sha256::digest(bytes);
        let actual = digest
            .iter()
            .fold(String::with_capacity(64), |mut acc, byte| {
                use std::fmt::Write;
                let _ = write!(acc, "{byte:02x}");
                acc
            });
        if actual == expected_sha256 {
            Ok(())
        } else {
            anyhow::bail!("manifest sha mismatch")
        }
    }
}
