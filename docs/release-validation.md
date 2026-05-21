# Release Validation

Run these checks before cutting a Nexara release.

## Local Preflight

```bash
cargo fmt --all -- --check
cargo test --workspace
./scripts/release-checklist.sh preflight
```

Expected result: all commands exit successfully.

Optional dependency policy checks:

```bash
./scripts/release-checklist.sh security
```

This mirrors the repository security workflow and `deny.toml` policy when the
tools are installed locally.

## Crate Checks

Run focused tests when a release touches the relevant crate:

```bash
cargo test -p nexara-core
cargo test -p nexara-registry
cargo test -p nexara-secrets
cargo test -p nexara-remote
cargo test -p nexara-runtime
cargo test -p nexara-learning
cargo test -p nexara-learning-memory
cargo test -p nexara-learning-jsonl
cargo test -p nexara-learning-sled
cargo test -p nexara-compat-qwen
cargo test -p nexara-server
```

## Packaging Dry Run

When publication metadata and dependency order are finalized, inspect the plan
and run the publish dry run:

```bash
./scripts/release-checklist.sh publish-plan
./scripts/release-checklist.sh dry-run-publish all
```

The script packages and publishes crates in dependency order:

1. `nexara-core`
2. `nexara-secrets`
3. `nexara-registry`
4. `nexara-remote`
5. `nexara-runtime`
6. `nexara-learning`
7. `nexara-learning-memory`
8. `nexara-learning-jsonl`
9. `nexara-learning-sled`
10. `nexara-compat-qwen`
11. `nexara-server`
12. `nexara`

For staged checks, use `foundation`, `contracts`, `learning`, `compat`,
`server`, and `facade`.

## Release Candidate

Before tagging:

```bash
./scripts/release-checklist.sh release-candidate
./scripts/release-checklist.sh sdk-package
```

The release-candidate phase runs serial workspace tests, checks the JavaScript
SDK package, verifies required OSS and docs files, and confirms the public docs
mention `host_requirements`.

## Release Notes

Before tagging:

- Update `CHANGELOG.md` with Added, Changed, Fixed, and Security sections where applicable.
- Confirm `README.md` examples match the public API.
- Confirm `SECURITY.md` contains the current private reporting channel.
- Confirm docs mention `host_requirements`, not legacy app-specific compatibility fields.
- Confirm host-owned adapter behavior is documented outside the standalone Nexara library.
