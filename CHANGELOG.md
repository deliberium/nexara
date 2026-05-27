# Changelog

All notable changes to Nexara are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows semantic versioning once public releases begin.

## [Unreleased]

### Added

- None yet.

### Changed

- None yet.

### Fixed

- None yet.

## [0.1.1] - 2026-05-27

### Added

- Opt-in `nexara-policy` crate for deterministic one-line policy authoring.
- Inspectable `PolicyContract` rules, selectors, diagnostics, evaluations, and matched-rule metadata in `nexara-core`.
- Semantic tool capability and effect metadata for policy-level resource/action enforcement.
- Runtime policy contract enforcement with deny-by-default behavior for tools missing semantic capability metadata.
- Signed policy contract helpers for enterprise review workflows.
- `nexara-cli` crate with policy compile, simulate, explain, sign, and verify commands.
- `policy-shop` demo for `reads orders, never refunds`.
- Policy authoring documentation and threat-model coverage.

### Changed

- Updated release packaging and publish-order validation for the new policy and CLI crates.
- Updated Nexara website and architecture diagrams to show one-line policy authoring and runtime enforcement.

### Fixed

- Runtime policy denials now emit audit records with matched-rule policy context.

## [0.1.0] - 2026-05-22

### Added

- Initial public Nexara workspace with a host-neutral facade crate.
- `nexara-core` crate with tool descriptors, call requests, call results, trust tiers, action classes, confirmation policy, registry, broker, descriptor validation, audit records, and shared errors.
- `nexara-registry` crate with signed skill index models, skill manifests, product-neutral `host_requirements`, manifest parsing, manifest SHA validation, file/HTTP index sources, and JSON installed-skill storage.
- `nexara-secrets` crate with sync and async secret-store traits, redacted `SecretValue`, memory storage, and an encrypted-file implementation.
- `nexara-remote` crate with product-neutral remote endpoint modeling, bearer auth providers, host allowlist validation, diagnostics, and HTTP list/call client helpers.
- `nexara-runtime` crate with host-provided catalogs, host executors, trust resolvers, tool selection, policy enforcement, payload limits, concurrency limits, confirmation checks, and audit emission.
- `nexara-learning` crate with bounded usage-signal event types, score adjustment configuration, and explainable learned ranking influence.
- `nexara-learning-memory`, `nexara-learning-jsonl`, and `nexara-learning-sled` storage crates for ephemeral, inspectable local, and embedded durable usage signals.
- `nexara-compat-qwen` crate for Qwen-family prompt profile detection and Qwen-style `mcpServers` config import.
- `nexara-server` crate with authenticated Axum routes for health, tool listing, tool calls, installed-skill listing, and admin skill install.
- JavaScript SDK package with typed `listTools()` and `callTool()` helpers.
- GitHub pull request template, Copilot guidance, CI workflow, release checklist, governance docs, and security policy.
- Architecture documentation with component boundaries for core, registry, remote, secrets, runtime, learning, server, and host adapters.
- Guidance for replacing legacy app-specific compatibility fields with `host_requirements`.
- Secure-by-default server authentication with explicit development no-auth mode.
- Secret redaction semantics so secret values do not implement debug-style exposure.
- Policy boundaries for action class, trust tier, confirmation, payload size, concurrency, and host allowlists.
- Signed-index fixture matrix covering valid, tampered, unsupported-host, missing-secret, and confirmation-required flows.
- Expanded Rust and JavaScript validation for remote clients, server routes, registry sources, runtime policy, facade feature exports, and SDK request behavior.
- Threat-model documentation covering signed registries, remote tools, secrets, audit, and learned usage signals.
- Release checklist for package verification and downstream host adapter validation.
- Public documentation pass for architecture, deployment, release validation, skill manifests, remote protocol, and threat model.

[Unreleased]: https://github.com/deliberium/nexara/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/deliberium/nexara/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/deliberium/nexara/releases/tag/v0.1.0
