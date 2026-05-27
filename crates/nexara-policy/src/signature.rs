use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use nexara_core::{PolicyContract, PolicyDefaults, PolicyRule, PolicySource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedPolicyContract {
    pub schema_version: String,
    pub contract: PolicyContract,
    pub reviewer: PolicyContractReviewer,
    pub signature: PolicyContractSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyContractReviewer {
    pub id: String,
    pub display_name: String,
    pub public_key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyContractSignature {
    pub algorithm: String,
    pub key_id: String,
    pub signature: String,
}

#[derive(Serialize)]
struct SignedPolicyContractPayload<'a> {
    schema_version: &'a str,
    contract: PolicyContractSignaturePayload<'a>,
    reviewer: &'a PolicyContractReviewer,
}

#[derive(Serialize)]
struct PolicyContractSignaturePayload<'a> {
    id: &'a str,
    version: &'a str,
    source: &'a PolicySource,
    rules: &'a [PolicyRule],
    defaults: &'a PolicyDefaults,
    created_at: &'a Option<String>,
}

pub fn signed_policy_contract_payload_bytes(
    schema_version: &str,
    contract: &PolicyContract,
    reviewer: &PolicyContractReviewer,
) -> Result<Vec<u8>> {
    serde_json::to_vec(&SignedPolicyContractPayload {
        schema_version,
        contract: PolicyContractSignaturePayload {
            id: &contract.id,
            version: &contract.version,
            source: &contract.source,
            rules: &contract.rules,
            defaults: &contract.defaults,
            created_at: &contract.created_at,
        },
        reviewer,
    })
    .context("failed encoding signed policy contract payload")
}

pub fn verify_signed_policy_contract(
    signed: &SignedPolicyContract,
    public_key_b64: &str,
) -> Result<()> {
    if signed.schema_version != "1" {
        bail!(
            "unsupported signed policy contract schema version '{}'",
            signed.schema_version
        );
    }
    if signed.signature.algorithm != "ed25519" {
        bail!(
            "unsupported signed policy contract signature algorithm '{}'",
            signed.signature.algorithm
        );
    }
    if signed.signature.key_id != signed.reviewer.public_key_id {
        bail!("signed policy contract key_id must match reviewer public_key_id");
    }
    let public_key_bytes = BASE64
        .decode(public_key_b64.trim())
        .context("failed decoding policy contract public key")?;
    let public_key_array: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("policy contract public key must be 32 bytes"))?;
    let public_key = VerifyingKey::from_bytes(&public_key_array)
        .context("failed parsing policy contract public key")?;
    let signature_bytes = BASE64
        .decode(signed.signature.signature.trim())
        .context("failed decoding policy contract signature")?;
    let signature = Signature::from_slice(&signature_bytes)
        .context("failed parsing policy contract signature")?;
    let payload = signed_policy_contract_payload_bytes(
        &signed.schema_version,
        &signed.contract,
        &signed.reviewer,
    )?;
    public_key
        .verify(&payload, &signature)
        .map_err(|_| anyhow::anyhow!("signed policy contract signature verification failed"))
}
