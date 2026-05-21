# Copilot Instructions for Nexara

Nexara is a contract-first capability runtime for AI systems. Keep it
host-neutral: public crates should model reusable contracts, policy, discovery,
execution boundaries, and audit behavior without naming or assuming any specific
host application.

## Build, Test, And Lint

- Build the Rust workspace: `cargo build --workspace`
- Run the full Rust test suite: `cargo test --workspace`
- Check formatting: `cargo fmt --all -- --check`
- Run focused crate tests:
  - `cargo test -p nexara-core`
  - `cargo test -p nexara-registry`
  - `cargo test -p nexara-secrets`
  - `cargo test -p nexara-remote`
  - `cargo test -p nexara-runtime`
  - `cargo test -p nexara-learning`
  - `cargo test -p nexara-compat-qwen`
  - `cargo test -p nexara-server`
- Run the release preflight: `./scripts/release-checklist.sh preflight`
- Validate the JavaScript SDK package shape from `sdk/javascript` once npm metadata changes: `npm pack --dry-run`

## High-Level Architecture

- `nexara-core` defines the shared domain language: tool descriptors, tool call
  requests/results, broker requests, action classes, trust tiers, confirmation
  policy, audit records, descriptor validation, `ToolRegistry`, `ToolBroker`,
  shared errors, and `NEXARA_VERSION`.
- `nexara-registry` owns signed skill index and manifest contracts. It uses
  product-neutral `host_requirements`; do not introduce app-specific
  compatibility fields into Nexara schemas, fixtures, examples, or parsers.
- `nexara-secrets` owns redacted `SecretValue`, sync `SecretStore`, async
  `AsyncSecretStore`, memory storage, and encrypted-file storage. Secret values
  must not leak through `Debug`, descriptors, audit payloads, learned records,
  or prompt-facing strings.
- `nexara-remote` owns remote endpoint modeling, auth providers, allowlist
  checks, HTTP discovery/call behavior, diagnostics, and circuit-breaker state.
  Imported endpoints are candidates; hosts still decide trust and enablement.
- `nexara-runtime` is the policy enforcement point. It combines catalog
  providers, host executors, trust resolvers, broker selection, payload limits,
  confirmation checks, concurrency limits, lifecycle hooks, and audit emission.
- `nexara-learning` owns optional usage-signal events and bounded ranking
  adjustments. Storage lives in `nexara-learning-memory`,
  `nexara-learning-jsonl`, `nexara-learning-sled`, or a host-provided store.
- `nexara-compat-qwen` imports Qwen-family MCP configuration into product-neutral
  endpoint records without bypassing Nexara policy.
- `nexara-server` wraps runtime and registry contracts in an authenticated Axum
  surface. It owns secure defaults for standalone use, while host applications
  may call lower-level contracts directly.
- `crates/nexara` is the facade crate. Keep feature flags precise so hosts can
  avoid pulling server, storage, or compatibility dependencies they do not use.
- `sdk/javascript` is a small HTTP client reference, not a replacement for the
  Rust contract surface.

## Key Conventions

- Preserve the product boundary from `docs/architecture.md`: Nexara provides
  contracts and enforcement; hosts provide identity, execution context, built-in
  tools, environment parsing, deployment choices, and final side effects.
- Do not add host-specific public fields to Nexara. If a host needs
  compatibility metadata, put it under `host_requirements` or a host-owned
  adapter.
- Keep policy checks centralized in `nexara-runtime`. Broker ranking, Qwen
  import, remote discovery, learning signals, and registry metadata must never
  bypass runtime policy.
- Keep descriptors prompt-safe and bounded.
- Prefer typed structs for public contracts. Use `serde_json::Value` only for
  payloads that are intentionally host/tool-defined.
- Treat learned signals as advisory. They may adjust ranking inside configured
  bounds, but cannot override scope, action class, trust tier, confirmation,
  host allowlists, server auth, or payload/concurrency limits.
- Support sync and async contracts where host implementations may be local,
  remote, slow, or backed by a vault/KMS/keychain.
- Keep server auth secure by default. Development no-auth must remain explicit
  and obvious in code and docs.
- Keep Qwen compatibility generic. Do not import host-specific CLI review state,
  catalog URLs, or approval workflows into `nexara-compat-qwen`.
- Keep examples and docs aligned with actual implementation.

## Public API Change Rules

When changing tool descriptors, call requests, call results, policy enums, or
broker behavior, update:

- `nexara-core` types and tests
- `nexara-runtime` policy flow when enforcement is affected
- `nexara-server` response/request mappings if HTTP behavior changes
- `sdk/javascript` if the HTTP wire contract changes
- `README.md`, relevant crate README files, and docs
- `CHANGELOG.md`

When changing skill registry behavior, update:

- `nexara-registry` manifest/index types, parser, validation, and fixtures
- `docs/skill-manifest-spec.md`
- `docs/threat-model.md` when trust or verification changes

When changing server routes, update:

- `nexara-server` route tests
- `sdk/javascript` if the client surface changes
- `docs/deployment.md`
- `docs/release-validation.md`
- server auth and admin API notes in `SECURITY.md`

## Security And Privacy Rules

- Never place raw secrets in descriptors, prompts, audit records, learned usage
  records, diagnostics, or error strings.
- Never make remote host allowlists implicit. Empty allowlists may be acceptable
  only when explicitly documented as a host decision for local/development use.
- Do not make admin APIs available without auth by default.
- Keep signature verification and manifest hash validation deterministic and
  covered by tests.
- Do not let imported Qwen/MCP entries become enabled tools without host review.
- Audit records should identify actions and outcomes without persisting large or
  sensitive raw tool results.

## Documentation Tone

Nexara docs should read like a reusable library, not an internal subsystem of a
larger product. Prefer concrete host-neutral examples and clearly mark any
host-owned behavior as adapter responsibility.
