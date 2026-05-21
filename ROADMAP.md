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

## v0.2

- Remote health-aware broker inputs and richer diagnostics.
- Pluggable registry cache interfaces for large catalogs.
- Host-provided key-management contracts for encrypted secret stores.
- WASM component runtime exploration behind strict policy gates.
- MCP HTTP and stdio transport import where host sandboxing can be explicit.
- Benchmarks for broker selection, registry load, HTTP roundtrip, and learned signal storage.

## Later

- Local process sandboxing patterns for hosts that want managed execution.
- Multi-tenant server profiles and tenant-scoped admin APIs.
- Policy simulation tools for registry review.
- Optional UI components for human approval and skill readiness flows.
