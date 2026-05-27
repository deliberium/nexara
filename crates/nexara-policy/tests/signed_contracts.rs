use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::{Signer, SigningKey};
use nexara_core::{PolicyDiagnostic, PolicyDiagnosticCode, PolicyDiagnosticSeverity};
use nexara_policy::{
    PolicyCompileOptions, PolicyCompiler, PolicyContractReviewer, PolicyContractSignature,
    SignedPolicyContract, signed_policy_contract_payload_bytes, verify_signed_policy_contract,
};

#[test]
fn signed_policy_contract_verifies_and_rejects_tampering() {
    let compiled = PolicyCompiler::default()
        .compile(
            "reads orders, never refunds",
            PolicyCompileOptions::default(),
        )
        .unwrap();
    let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
    let reviewer = PolicyContractReviewer {
        id: "security-review".to_string(),
        display_name: "Security Review".to_string(),
        public_key_id: "policy-key-1".to_string(),
    };
    let payload = signed_policy_contract_payload_bytes("1", &compiled.policy, &reviewer).unwrap();
    let signature = signing_key.sign(&payload);
    let signed = SignedPolicyContract {
        schema_version: "1".to_string(),
        contract: compiled.policy.clone(),
        reviewer,
        signature: PolicyContractSignature {
            algorithm: "ed25519".to_string(),
            key_id: "policy-key-1".to_string(),
            signature: BASE64.encode(signature.to_bytes()),
        },
    };
    let public_key = BASE64.encode(signing_key.verifying_key().to_bytes());

    verify_signed_policy_contract(&signed, &public_key).unwrap();

    let mut diagnostics_tampered = signed.clone();
    diagnostics_tampered
        .contract
        .diagnostics
        .push(PolicyDiagnostic {
            severity: PolicyDiagnosticSeverity::Info,
            code: PolicyDiagnosticCode::MissingCatalogMapping,
            message: "review-only diagnostic changed after signing".to_string(),
        });
    verify_signed_policy_contract(&diagnostics_tampered, &public_key).unwrap();

    let mut tampered = signed;
    tampered.contract.rules[0].reason = "tampered".to_string();
    assert!(verify_signed_policy_contract(&tampered, &public_key).is_err());
}
