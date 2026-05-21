# Contributing

Thank you for helping improve Nexara. The project is young, so the best
contributions are small, well-tested changes that strengthen the public
contracts without making host applications depend on product-specific
assumptions.

## Development Setup

Install a Rust toolchain that satisfies the workspace MSRV in `Cargo.toml`.

```bash
rustup toolchain install stable
cargo test --workspace
```

The workspace has no required network services for the default test suite.

## Before Opening A Pull Request

Run:

```bash
cargo fmt --all -- --check
cargo test --workspace
```

When touching public API, also update the relevant crate README, top-level
README, docs page, and changelog entry.

## Design Principles

- Keep host-neutral. CLIs, IDEs, chat surfaces, and local agents
  should all be able to use the same core contracts.
- Keep policy enforcement centralized in `nexara-runtime`; helper crates can
  describe capability metadata but must not bypass runtime checks.
- Prefer typed contracts over unstructured `serde_json::Value` for public APIs.
- Keep descriptors prompt-safe and bounded. Descriptors must never include
  secrets, raw credentials, or large opaque payloads.
- Treat learned usage signals as suggestions. They may influence ranking but
  must never override action, trust, scope, confirmation, allowlist, or auth
  policy.
- Support both sync and async host integrations where the contract naturally
  crosses process, network, or secret-store boundaries.

## Testing Expectations

Add or update tests when you change:

- descriptor validation
- broker ranking or learned adjustments
- registry signature, manifest, or storage behavior
- remote allowlists, auth, diagnostics, or call behavior
- runtime policy decisions
- server auth or admin routes
- secret-store redaction or persistence

Use focused crate tests for narrow changes and workspace tests for changes that
cross crate boundaries.

## Documentation

Documentation should explain the host contract and safety model. Avoid writing
product-specific examples unless the page is explicitly about generic host
migration. Prefer examples that can be copied into another host application.

## Release Checklist

Before tagging a release:

```bash
./scripts/release-checklist.sh preflight
```

Then follow [docs/release-validation.md](docs/release-validation.md).
