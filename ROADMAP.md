# Roadmap

This roadmap is directional. Security, compatibility, and host-integration
needs can change the order.

## v0.1.0

- Public Rust workspace with `nexara` facade and focused crates.
- Core tool descriptors, action classes, trust tiers, broker, registry, audit, and validation.
- Signed skill registry contracts with product-neutral `host_requirements`.
- Sync and async secret-store contracts with memory and encrypted-file implementations.
- Remote HTTP discovery and execution contracts with auth, allowlists, and diagnostics.
- Runtime policy enforcement with host-provided catalogs, executors, and trust resolvers.
- Optional learning layer with memory, JSONL, sled, and host-provided storage choices.
- Qwen-family MCP config import in `nexara-compat-qwen`.
- Authenticated `nexara-server` list/call routes and admin install, enable/disable, readiness, secret, and metrics APIs.
- Runnable embedded, HTTP server, custom index, remote discovery, and learning examples.
- Signed-index fixtures for tampered, unsupported, missing-secret, and confirmation-required skill flows.
- Security workflow using `cargo audit` and documented local fallbacks.
- JavaScript SDK for list/call and admin flows.
- Host-neutral adapter contracts for catalogs, execution, trust resolution, and `host_requirements`.

## v0.1.x

- Add packaging dry-run workflow once crate publication order is finalized.
- Expand conformance fixtures for additional host requirement and registry cache edge cases.
- Add richer examples for custom auth, KMS-backed secrets, and multi-user learned signals.
- Add opt-in `nexara-policy` for deterministic one-line policy authoring: plain intent such as `reads orders, never refunds` compiles into inspectable policy contracts.
- Add semantic capability/effect metadata to descriptors and manifests so policy contracts can distinguish resource operations such as `orders.read` and `orders.refunds.create`.
- Add catalog simulation and runtime policy explanations for allow, deny, confirmation, and missing-capability decisions.
- Add signed policy contracts for enterprise review workflows.
- Add `nexara-cli` with policy compile, simulate, explain, sign, and verify commands.
- Add the `policy-shop` demo and update the Nexara website with an interactive one-line policy demo.

### Policy Authoring Non-Goals For The First Implementation

The first policy-authoring implementation is intentionally narrow so the core
loop stays demoable, inspectable, and enforceable. These are not in scope:

- Free-form LLM policy authoring.
- Legal/compliance policy generation.
- Cross-language policy SDKs beyond stable JSON output.
- Full ABAC/RBAC engine.
- Full UI policy editor in the Rust repository.
- Automatic trust of remote server capability labels without signed registry or host review.

## v0.2

- Remote health-aware broker inputs and richer diagnostics.
- Pluggable registry cache interfaces for large catalogs.
- Host-provided key-management contracts for encrypted secret stores.
- WASM component runtime exploration behind strict policy gates.
- MCP HTTP and stdio transport import where host sandboxing can be explicit.
- Benchmarks for broker selection, registry load, HTTP roundtrip, and learned signal storage.
- Policy contract simulation attestations for teams that want to sign both the contract and reviewed catalog simulation output.

## Later

- Local process sandboxing patterns for hosts that want managed execution.
- Multi-tenant server profiles and tenant-scoped admin APIs.
- Optional UI components for human approval and skill readiness flows.
