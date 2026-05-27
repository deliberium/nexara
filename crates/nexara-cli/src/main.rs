use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::{Signer, SigningKey};
use nexara_core::{
    ActionClass, CapabilitySensitivity, ToolCapability, ToolDescriptor, ToolRegistry, TrustTier,
};
use nexara_policy::{
    PolicyCompileOptions, PolicyCompiler, PolicyContractReviewer, PolicyContractSignature,
    SignedPolicyContract, signed_policy_contract_payload_bytes, simulate_catalog,
    verify_signed_policy_contract,
};
use std::collections::BTreeMap;
use std::fs;

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [command, subcommand, rest @ ..] if command == "policy" => match subcommand.as_str() {
            "compile" => compile_policy(rest),
            "explain" => explain_policy(rest),
            "sign" => sign_policy(rest),
            "verify" => verify_policy(rest),
            "simulate" => simulate_policy(rest),
            _ => bail!("unknown policy subcommand '{subcommand}'"),
        },
        [command, subcommand] if command == "demo" && subcommand == "shop" => {
            println!("Run: cargo run -p policy-shop -- \"reads orders, never refunds\"");
            Ok(())
        }
        _ => bail!(
            "usage: nexara policy compile <intent> [--json] | policy sign <contract.json> --key-id <id> --key-file <path> | policy verify <signed.json> --trusted-keys <path>"
        ),
    }
}

fn compile_policy(args: &[String]) -> Result<()> {
    let json = args.iter().any(|arg| arg == "--json");
    let intent = args
        .iter()
        .find(|arg| !arg.starts_with("--"))
        .context("policy intent is required")?;
    let compiled = PolicyCompiler::default().compile(intent, PolicyCompileOptions::default())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&compiled.policy)?);
    } else {
        println!("{}", compiled.explain_markdown());
    }
    Ok(())
}

fn explain_policy(args: &[String]) -> Result<()> {
    let path = args.first().context("contract path is required")?;
    let raw = fs::read_to_string(path).with_context(|| format!("failed reading '{path}'"))?;
    let contract = serde_json::from_str(&raw).context("failed parsing policy contract")?;
    println!(
        "{}",
        nexara_policy::CompiledPolicy { policy: contract }.explain_markdown()
    );
    Ok(())
}

fn sign_policy(args: &[String]) -> Result<()> {
    let path = args.first().context("contract path is required")?;
    let key_id = option_value(args, "--key-id").context("--key-id is required")?;
    let key_file = option_value(args, "--key-file").context("--key-file is required")?;
    let raw = fs::read_to_string(path).with_context(|| format!("failed reading '{path}'"))?;
    let contract = serde_json::from_str(&raw).context("failed parsing policy contract")?;
    let seed = fs::read(key_file).with_context(|| format!("failed reading '{key_file}'"))?;
    let seed: [u8; 32] = seed
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("policy signing key file must contain 32 raw bytes"))?;
    let signing_key = SigningKey::from_bytes(&seed);
    let reviewer = PolicyContractReviewer {
        id: key_id.to_string(),
        display_name: key_id.to_string(),
        public_key_id: key_id.to_string(),
    };
    let payload = signed_policy_contract_payload_bytes("1", &contract, &reviewer)?;
    let signature = signing_key.sign(&payload);
    let signed = SignedPolicyContract {
        schema_version: "1".to_string(),
        contract,
        reviewer,
        signature: PolicyContractSignature {
            algorithm: "ed25519".to_string(),
            key_id: key_id.to_string(),
            signature: BASE64.encode(signature.to_bytes()),
        },
    };
    println!("{}", serde_json::to_string_pretty(&signed)?);
    Ok(())
}

fn simulate_policy(args: &[String]) -> Result<()> {
    let policy_path = args.first().context("policy contract path is required")?;
    let catalog_path = option_value(args, "--catalog").context("--catalog is required")?;
    let policy_raw = fs::read_to_string(policy_path)
        .with_context(|| format!("failed reading '{policy_path}'"))?;
    let policy = serde_json::from_str(&policy_raw).context("failed parsing policy contract")?;
    let catalog_raw = fs::read_to_string(catalog_path)
        .with_context(|| format!("failed reading '{catalog_path}'"))?;
    let registry = registry_from_catalog_json(&catalog_raw)?;
    let simulation = simulate_catalog(&policy, &registry);
    println!("{}", serde_json::to_string_pretty(&simulation)?);
    Ok(())
}

fn verify_policy(args: &[String]) -> Result<()> {
    let path = args.first().context("signed contract path is required")?;
    let trusted_keys =
        option_value(args, "--trusted-keys").context("--trusted-keys is required")?;
    let raw = fs::read_to_string(path).with_context(|| format!("failed reading '{path}'"))?;
    let signed: SignedPolicyContract =
        serde_json::from_str(&raw).context("failed parsing signed policy contract")?;
    let keys_raw = fs::read_to_string(trusted_keys)
        .with_context(|| format!("failed reading '{trusted_keys}'"))?;
    let keys: BTreeMap<String, String> =
        serde_json::from_str(&keys_raw).context("failed parsing trusted keys")?;
    let public_key = keys
        .get(&signed.signature.key_id)
        .context("trusted key is not configured")?;
    verify_signed_policy_contract(&signed, public_key)?;
    println!("signed policy contract verified");
    Ok(())
}

fn option_value<'a>(args: &'a [String], option: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == option)
        .map(|window| window[1].as_str())
}

fn registry_from_catalog_json(raw: &str) -> Result<ToolRegistry> {
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(raw).context("failed parsing catalog JSON")?;
    let mut registry = ToolRegistry::new();
    for entry in entries {
        let name = entry
            .get("tool")
            .or_else(|| entry.get("name"))
            .and_then(|value| value.as_str())
            .context("catalog entry requires tool or name")?;
        let capability = entry
            .get("capability")
            .and_then(|value| value.as_str())
            .context("catalog entry requires capability")?;
        let segments = capability.split('.').collect::<Vec<_>>();
        if segments.len() < 2 {
            bail!("capability '{capability}' must contain resource and operation");
        }
        let operation = segments.last().unwrap().to_string();
        let resource_path = segments[..segments.len() - 1]
            .iter()
            .map(|segment| (*segment).to_string())
            .collect::<Vec<_>>();
        let action_class = match operation.as_str() {
            "read" | "search" | "export" => ActionClass::Read,
            "execute" => ActionClass::Execute,
            _ => ActionClass::Write,
        };
        registry.register(ToolDescriptor {
            name: name.to_string(),
            description: name.to_string(),
            tags: Vec::new(),
            scopes: vec!["read".to_string()],
            trust_tier: TrustTier::RemoteNexara,
            action_class,
            enabled: true,
            input_schema: None,
            guidance: None,
            capabilities: vec![ToolCapability {
                id: capability.to_string(),
                display_name: Some(capability.replace('.', " ")),
                description: Some(format!("Catalog capability {capability}")),
                resource: resource_path[0].clone(),
                resource_path,
                operation,
                action_class,
                sensitivity: CapabilitySensitivity::Business,
                aliases: Vec::new(),
            }],
            effects: Vec::new(),
        });
    }
    Ok(registry)
}
