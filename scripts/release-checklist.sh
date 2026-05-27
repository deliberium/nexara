#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
manifest_path="${NEXARA_MANIFEST_PATH:-$repo_root/Cargo.toml}"
publish_attempts="${NEXARA_PUBLISH_ATTEMPTS:-8}"
publish_initial_wait_seconds="${NEXARA_PUBLISH_INITIAL_WAIT_SECONDS:-20}"
publish_max_wait_seconds="${NEXARA_PUBLISH_MAX_WAIT_SECONDS:-300}"

phase="${1:-help}"

publish_order=(
  nexara-core
  nexara-secrets
  nexara-registry
  nexara-remote
  nexara-runtime
  nexara-policy
  nexara-learning
  nexara-learning-memory
  nexara-learning-jsonl
  nexara-learning-sled
  nexara-compat-qwen
  nexara-server
  nexara-cli
  nexara
)

run() {
  printf '\n==> %s\n' "$*"
  "$@"
}

print_help() {
  cat <<EOF
Usage: $(basename "$0") <phase>

Phases:
  preflight         Run fmt, clippy, and workspace tests.
  release-candidate Run serial workspace tests, SDK package checks, docs checks, and metadata checks.
  security          Run cargo audit and cargo deny check.
  sdk-package       Validate the JavaScript SDK package shape with npm pack --dry-run.
  sdk-publish       Publish the JavaScript SDK to npm.
  bump-version      Update workspace crate versions, internal crate dependencies, and SDK package version.
  foundation        Package crates with no Nexara crate dependencies.
  contracts         Package core contracts and runtime-adjacent crates.
  learning          Package learned-usage storage crates.
  policy            Package policy authoring and CLI crates.
  compat            Package compatibility import crates.
  server            Package nexara-server.
  facade            Package the nexara facade crate.
  dry-run-publish   Run cargo publish --dry-run for foundation, contracts, learning, compat, server, facade, all, or one crate.
  publish           Build, verify, and publish crates to crates.io in dependency order.
  publish-plan      Print the recommended publish order and commands.
  help              Show this help.

Notes:
  - Workspace manifest: $manifest_path
  - foundation can package before any Nexara crates are published.
  - later phases require earlier crates to exist on crates.io for Cargo publish verification.
  - dry-run-publish all continues past expected publish-order dependency gaps and reports skipped crates.
  - publish accepts a target: foundation, contracts, learning, compat, server, facade, all, or one crate name.
  - sdk-publish uses npm credentials from the caller's environment.
  - publish retries crates.io 429/index propagation failures and skips versions already published.
  - Tune publish retry behavior with NEXARA_PUBLISH_ATTEMPTS, NEXARA_PUBLISH_INITIAL_WAIT_SECONDS, and NEXARA_PUBLISH_MAX_WAIT_SECONDS.
EOF
}

bump_version() {
  local new_version="${2:-}"
  local cargo_tomls=()
  local manifest

  if [[ -z "$new_version" ]]; then
    printf 'Usage: %s bump-version <semver>\n' "$(basename "$0")" >&2
    exit 1
  fi

  if [[ ! "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
    printf 'Invalid version: %s\n' "$new_version" >&2
    printf 'Expected a semver value like 0.1.1 or 0.2.0-rc.1\n' >&2
    exit 1
  fi

  while IFS= read -r manifest; do
    cargo_tomls+=("$manifest")
  done < <(find "$repo_root" \( -path "$repo_root/target" -o -path "$repo_root/.git" \) -prune -o -name Cargo.toml -type f -print | sort)

  if [[ ${#cargo_tomls[@]} -eq 0 ]]; then
    printf 'No Cargo.toml files found under %s\n' "$repo_root" >&2
    exit 1
  fi

  for manifest in "${cargo_tomls[@]}"; do
    run perl -0pi -e 's/^version = "[^"]+"$/version = "'"$new_version"'"/m' "$manifest"
    run perl -0pi -e 's/(nexara(?:-[a-z0-9]+)*\s*=\s*\{[^\n]*version\s*=\s*")[^"]+("[^\n]*\})/${1}'"$new_version"'${2}/g' "$manifest"
  done

  if [[ -f "$repo_root/sdk/javascript/package.json" ]]; then
    run perl -0pi -e 's/"version":\s*"[^"]+"/"version": "'"$new_version"'"/' "$repo_root/sdk/javascript/package.json"
  fi
}

preflight() {
  run cargo fmt --manifest-path "$manifest_path" --all --check
  run cargo clippy --manifest-path "$manifest_path" --workspace --all-targets
  run cargo test --manifest-path "$manifest_path" --workspace
  run cargo test --manifest-path "$manifest_path" -p nexara --all-features
  if [[ -f "$repo_root/sdk/javascript/package.json" ]]; then
    (cd "$repo_root/sdk/javascript" && run npm test)
  fi
}

sdk_package() {
  if [[ -f "$repo_root/sdk/javascript/package.json" ]]; then
    mkdir -p "$repo_root/target/npm-cache"
    (
      cd "$repo_root/sdk/javascript"
      run npm --cache "$repo_root/target/npm-cache" pack --dry-run
    )
  else
    printf '\n==> skipping JavaScript SDK package check: sdk/javascript/package.json not found\n'
  fi
}

sdk_publish() {
  sdk_package
  if [[ ! -f "$repo_root/sdk/javascript/package.json" ]]; then
    printf 'Cannot publish JavaScript SDK: sdk/javascript/package.json not found\n' >&2
    exit 1
  fi
  mkdir -p "$repo_root/target/npm-cache"
  (
    cd "$repo_root/sdk/javascript"
    run npm --cache "$repo_root/target/npm-cache" publish --access public
  )
}

require_release_files() {
  local required_files=(
    "$repo_root/README.md"
    "$repo_root/CHANGELOG.md"
    "$repo_root/CONTRIBUTING.md"
    "$repo_root/SECURITY.md"
    "$repo_root/ROADMAP.md"
    "$repo_root/AGENTS.md"
    "$repo_root/LICENSE.md"
    "$repo_root/docs/architecture.md"
    "$repo_root/docs/assets/nexara-architecture.svg"
    "$repo_root/docs/deployment.md"
    "$repo_root/docs/qwen-compatibility.md"
    "$repo_root/docs/release-validation.md"
    "$repo_root/docs/remote-tool-protocol.md"
    "$repo_root/docs/skill-manifest-spec.md"
    "$repo_root/docs/threat-model.md"
    "$repo_root/docs/user-guide.md"
    "$repo_root/.github/workflows/ci.yml"
    "$repo_root/.github/workflows/security.yml"
    "$repo_root/deny.toml"
  )
  local path

  for path in "${required_files[@]}"; do
    run test -f "$path"
  done
}

release_candidate() {
  run cargo fmt --manifest-path "$manifest_path" --all --check
  run cargo clippy --manifest-path "$manifest_path" --workspace --all-targets
  run cargo test --manifest-path "$manifest_path" --workspace -- --test-threads=1
  run cargo test --manifest-path "$manifest_path" -p nexara --all-features
  if [[ -f "$repo_root/sdk/javascript/package.json" ]]; then
    (cd "$repo_root/sdk/javascript" && run npm test)
  fi
  sdk_package
  require_release_files
  run grep -Fq 'host_requirements' "$repo_root/README.md"
  run grep -Fq 'host_requirements' "$repo_root/docs/skill-manifest-spec.md"
  run grep -Fq 'nexara-architecture.svg' "$repo_root/README.md"
  run grep -Fq '[0.1.1]' "$repo_root/CHANGELOG.md"
}

security() {
  run cargo audit
  run cargo deny check --manifest-path "$manifest_path"
}

package_crate() {
  local crate_name="$1"
  run cargo package --manifest-path "$manifest_path" -p "$crate_name" --allow-dirty
}

dry_run_publish_crate() {
  local crate_name="$1"
  run cargo publish --dry-run --manifest-path "$manifest_path" -p "$crate_name" --allow-dirty
}

crate_manifest_path() {
  local crate_name="$1"

  case "$crate_name" in
    nexara-core)
      printf '%s\n' "$repo_root/crates/nexara-core/Cargo.toml"
      ;;
    nexara-secrets)
      printf '%s\n' "$repo_root/crates/nexara-secrets/Cargo.toml"
      ;;
    nexara-registry)
      printf '%s\n' "$repo_root/crates/nexara-registry/Cargo.toml"
      ;;
    nexara-remote)
      printf '%s\n' "$repo_root/crates/nexara-remote/Cargo.toml"
      ;;
    nexara-runtime)
      printf '%s\n' "$repo_root/crates/nexara-runtime/Cargo.toml"
      ;;
    nexara-policy)
      printf '%s\n' "$repo_root/crates/nexara-policy/Cargo.toml"
      ;;
    nexara-learning)
      printf '%s\n' "$repo_root/crates/nexara-learning/Cargo.toml"
      ;;
    nexara-learning-memory)
      printf '%s\n' "$repo_root/crates/nexara-learning-memory/Cargo.toml"
      ;;
    nexara-learning-jsonl)
      printf '%s\n' "$repo_root/crates/nexara-learning-jsonl/Cargo.toml"
      ;;
    nexara-learning-sled)
      printf '%s\n' "$repo_root/crates/nexara-learning-sled/Cargo.toml"
      ;;
    nexara-compat-qwen)
      printf '%s\n' "$repo_root/crates/nexara-compat-qwen/Cargo.toml"
      ;;
    nexara-server)
      printf '%s\n' "$repo_root/crates/nexara-server/Cargo.toml"
      ;;
    nexara-cli)
      printf '%s\n' "$repo_root/crates/nexara-cli/Cargo.toml"
      ;;
    nexara)
      printf '%s\n' "$repo_root/crates/nexara/Cargo.toml"
      ;;
    *)
      printf 'Unknown crate: %s\n' "$crate_name" >&2
      return 1
      ;;
  esac
}

crate_version() {
  local crate_name="$1"
  local crate_manifest

  crate_manifest="$(crate_manifest_path "$crate_name")"
  awk -F\" '/^version = / { print $2; exit }' "$crate_manifest"
}

publish_targets() {
  local target="$1"

  case "$target" in
    foundation)
      printf '%s\n' nexara-core nexara-secrets
      ;;
    contracts)
      printf '%s\n' nexara-registry nexara-remote nexara-runtime nexara-learning
      ;;
    policy)
      printf '%s\n' nexara-policy nexara-cli
      ;;
    learning)
      printf '%s\n' nexara-learning-memory nexara-learning-jsonl nexara-learning-sled
      ;;
    compat)
      printf '%s\n' nexara-compat-qwen
      ;;
    server)
      printf '%s\n' nexara-server
      ;;
    facade)
      printf '%s\n' nexara
      ;;
    all)
      printf '%s\n' "${publish_order[@]}"
      ;;
    nexara-core|nexara-secrets|nexara-registry|nexara-remote|nexara-runtime|nexara-policy|nexara-learning|nexara-learning-memory|nexara-learning-jsonl|nexara-learning-sled|nexara-compat-qwen|nexara-server|nexara-cli|nexara)
      printf '%s\n' "$target"
      ;;
    *)
      printf 'Unknown publish target: %s\n' "$target" >&2
      printf 'Expected one of: foundation, contracts, learning, policy, compat, server, facade, all, or a Nexara crate name\n' >&2
      return 1
      ;;
  esac
}

output_indicates_already_published() {
  local output_file="$1"
  grep -Eiq 'already (uploaded|exists)|is already uploaded|crate version .* is already' "$output_file"
}

output_indicates_retryable_publish_failure() {
  local output_file="$1"

  grep -Eiq '(^|[^0-9])429([^0-9]|$)|too many requests|rate limit|try again later|Retry-After|failed to get a 200 OK response|failed to get successful HTTP response' "$output_file" \
    || grep -Eiq 'no matching package named `nexara|failed to select a version for the requirement `nexara|candidate versions found which didn'\''t match' "$output_file"
}

retry_after_seconds() {
  local output_file="$1"
  local retry_after

  retry_after="$(grep -Eio 'retry-after: *[0-9]+' "$output_file" | awk '{ print $2 }' | tail -n 1)"
  if [[ -n "$retry_after" ]]; then
    printf '%s\n' "$retry_after"
  fi
}

wait_before_publish_retry() {
  local output_file="$1"
  local attempt="$2"
  local wait_seconds
  local retry_after

  retry_after="$(retry_after_seconds "$output_file")"
  if [[ -n "$retry_after" ]]; then
    wait_seconds="$retry_after"
  else
    wait_seconds=$((publish_initial_wait_seconds * attempt))
  fi

  if (( wait_seconds > publish_max_wait_seconds )); then
    wait_seconds="$publish_max_wait_seconds"
  fi

  printf '\n==> crates.io is not ready yet; retrying in %s seconds\n' "$wait_seconds"
  sleep "$wait_seconds"
}

validate_publish_retry_config() {
  if [[ ! "$publish_attempts" =~ ^[1-9][0-9]*$ ]]; then
    printf 'Invalid NEXARA_PUBLISH_ATTEMPTS: %s\n' "$publish_attempts" >&2
    printf 'Expected a positive integer.\n' >&2
    exit 1
  fi

  if [[ ! "$publish_initial_wait_seconds" =~ ^[1-9][0-9]*$ ]]; then
    printf 'Invalid NEXARA_PUBLISH_INITIAL_WAIT_SECONDS: %s\n' "$publish_initial_wait_seconds" >&2
    printf 'Expected a positive integer.\n' >&2
    exit 1
  fi

  if [[ ! "$publish_max_wait_seconds" =~ ^[1-9][0-9]*$ ]]; then
    printf 'Invalid NEXARA_PUBLISH_MAX_WAIT_SECONDS: %s\n' "$publish_max_wait_seconds" >&2
    printf 'Expected a positive integer.\n' >&2
    exit 1
  fi
}

publish_crate() {
  local crate_name="$1"
  local crate_version
  local output_file
  local attempt=1

  crate_version="$(crate_version "$crate_name")"
  package_crate "$crate_name"

  while (( attempt <= publish_attempts )); do
    output_file="$(mktemp)"
    printf '\n==> cargo publish --manifest-path %s -p %s --allow-dirty (attempt %s/%s)\n' "$manifest_path" "$crate_name" "$attempt" "$publish_attempts"

    if cargo publish --manifest-path "$manifest_path" -p "$crate_name" --allow-dirty >"$output_file" 2>&1; then
      cat "$output_file"
      rm -f "$output_file"
      printf '\n==> published %s %s\n' "$crate_name" "$crate_version"
      return 0
    fi

    if output_indicates_already_published "$output_file"; then
      cat "$output_file"
      rm -f "$output_file"
      printf '\n==> skipping %s %s: this version already exists on crates.io\n' "$crate_name" "$crate_version"
      return 0
    fi

    if output_indicates_retryable_publish_failure "$output_file" && (( attempt < publish_attempts )); then
      cat "$output_file"
      wait_before_publish_retry "$output_file" "$attempt"
      rm -f "$output_file"
      attempt=$((attempt + 1))
      continue
    fi

    cat "$output_file"
    rm -f "$output_file"
    printf '\n==> failed to publish %s %s after %s attempt(s)\n' "$crate_name" "$crate_version" "$attempt" >&2
    return 1
  done
}

dry_run_publish_crate_or_skip() {
  local crate_name="$1"
  shift
  local required_dependencies=("$@")
  local output_file
  local dependency

  output_file="$(mktemp)"
  printf '\n==> cargo publish --dry-run --manifest-path %s -p %s --allow-dirty\n' "$manifest_path" "$crate_name"
  if cargo publish --dry-run --manifest-path "$manifest_path" -p "$crate_name" --allow-dirty >"$output_file" 2>&1; then
    cat "$output_file"
    rm -f "$output_file"
    return 0
  fi

  for dependency in "${required_dependencies[@]}"; do
    if grep -Fq "no matching package named \`$dependency\` found" "$output_file" \
      || { grep -Fq "failed to select a version for the requirement \`$dependency =" "$output_file" \
        && grep -Fq "candidate versions found which didn't match" "$output_file"; }; then
      printf '\n==> skipping %s: crates.io does not yet have required dependency %s\n' "$crate_name" "$dependency"
      rm -f "$output_file"
      return 0
    fi
  done

  cat "$output_file"
  rm -f "$output_file"
  return 1
}

foundation() {
  package_crate nexara-core
  package_crate nexara-secrets
}

contracts() {
  package_crate nexara-registry
  package_crate nexara-remote
  package_crate nexara-runtime
  package_crate nexara-learning
}

learning() {
  package_crate nexara-learning-memory
  package_crate nexara-learning-jsonl
  package_crate nexara-learning-sled
}

policy() {
  package_crate nexara-policy
  package_crate nexara-cli
}

compat() {
  package_crate nexara-compat-qwen
}

server() {
  package_crate nexara-server
}

facade() {
  package_crate nexara
}

dry_run_publish() {
  local target="${2:-all}"
  case "$target" in
    foundation)
      dry_run_publish_crate nexara-core
      dry_run_publish_crate nexara-secrets
      ;;
    contracts)
      dry_run_publish_crate_or_skip nexara-registry nexara-core
      dry_run_publish_crate_or_skip nexara-remote nexara-core
      dry_run_publish_crate_or_skip nexara-runtime nexara-core
      dry_run_publish_crate_or_skip nexara-learning nexara-core
      ;;
    learning)
      dry_run_publish_crate_or_skip nexara-learning-memory nexara-learning
      dry_run_publish_crate_or_skip nexara-learning-jsonl nexara-learning
      dry_run_publish_crate_or_skip nexara-learning-sled nexara-learning
      ;;
    policy)
      dry_run_publish_crate_or_skip nexara-policy nexara-core
      dry_run_publish_crate_or_skip nexara-cli nexara-core nexara-policy
      ;;
    compat)
      dry_run_publish_crate_or_skip nexara-compat-qwen nexara-remote
      ;;
    server)
      dry_run_publish_crate_or_skip nexara-server nexara-core nexara-registry nexara-runtime nexara-secrets
      ;;
    facade)
      dry_run_publish_crate_or_skip nexara nexara-core nexara-registry nexara-remote nexara-secrets nexara-runtime nexara-learning nexara-learning-memory nexara-learning-jsonl nexara-learning-sled nexara-compat-qwen nexara-server
      ;;
    all)
      dry_run_publish_crate nexara-core
      dry_run_publish_crate nexara-secrets
      dry_run_publish_crate_or_skip nexara-registry nexara-core
      dry_run_publish_crate_or_skip nexara-remote nexara-core
      dry_run_publish_crate_or_skip nexara-runtime nexara-core
      dry_run_publish_crate_or_skip nexara-policy nexara-core
      dry_run_publish_crate_or_skip nexara-learning nexara-core
      dry_run_publish_crate_or_skip nexara-learning-memory nexara-learning
      dry_run_publish_crate_or_skip nexara-learning-jsonl nexara-learning
      dry_run_publish_crate_or_skip nexara-learning-sled nexara-learning
      dry_run_publish_crate_or_skip nexara-compat-qwen nexara-remote
      dry_run_publish_crate_or_skip nexara-server nexara-core nexara-registry nexara-runtime nexara-secrets
      dry_run_publish_crate_or_skip nexara-cli nexara-core nexara-policy
      dry_run_publish_crate_or_skip nexara nexara-core nexara-registry nexara-remote nexara-secrets nexara-runtime nexara-policy nexara-learning nexara-learning-memory nexara-learning-jsonl nexara-learning-sled nexara-compat-qwen nexara-server
      ;;
    nexara-core|nexara-secrets|nexara-registry|nexara-remote|nexara-runtime|nexara-policy|nexara-learning|nexara-learning-memory|nexara-learning-jsonl|nexara-learning-sled|nexara-compat-qwen|nexara-server|nexara-cli|nexara)
      dry_run_publish_crate "$target"
      ;;
    *)
      printf 'Unknown dry-run-publish target: %s\n' "$target" >&2
      printf 'Expected one of: foundation, contracts, learning, policy, compat, server, facade, all, or a Nexara crate name\n' >&2
      exit 1
      ;;
  esac
}

publish() {
  local target="${2:-all}"
  local crate_name
  local crates=()
  local target_file

  validate_publish_retry_config

  target_file="$(mktemp)"
  if ! publish_targets "$target" >"$target_file"; then
    rm -f "$target_file"
    exit 1
  fi

  while IFS= read -r crate_name; do
    crates+=("$crate_name")
  done <"$target_file"
  rm -f "$target_file"

  if [[ ${#crates[@]} -eq 0 ]]; then
    printf 'No crates selected for publish target: %s\n' "$target" >&2
    exit 1
  fi

  printf 'Publishing to crates.io in this order:\n'
  for crate_name in "${crates[@]}"; do
    printf '  - %s %s\n' "$crate_name" "$(crate_version "$crate_name")"
  done

  for crate_name in "${crates[@]}"; do
    publish_crate "$crate_name"
  done
}

publish_plan() {
  cat <<EOF
Recommended publish order:
  1.  cargo publish --manifest-path "$manifest_path" -p nexara-core
  2.  cargo publish --manifest-path "$manifest_path" -p nexara-secrets
  3.  cargo publish --manifest-path "$manifest_path" -p nexara-registry
  4.  cargo publish --manifest-path "$manifest_path" -p nexara-remote
  5.  cargo publish --manifest-path "$manifest_path" -p nexara-runtime
  6.  cargo publish --manifest-path "$manifest_path" -p nexara-policy
  7.  cargo publish --manifest-path "$manifest_path" -p nexara-learning
  8.  cargo publish --manifest-path "$manifest_path" -p nexara-learning-memory
  9.  cargo publish --manifest-path "$manifest_path" -p nexara-learning-jsonl
  10. cargo publish --manifest-path "$manifest_path" -p nexara-learning-sled
  11. cargo publish --manifest-path "$manifest_path" -p nexara-compat-qwen
  12. cargo publish --manifest-path "$manifest_path" -p nexara-server
  13. cargo publish --manifest-path "$manifest_path" -p nexara-cli
  14. cargo publish --manifest-path "$manifest_path" -p nexara

Recommended verification flow:
  $(basename "$0") preflight
  $(basename "$0") release-candidate
  $(basename "$0") security
  $(basename "$0") bump-version 0.1.1
  $(basename "$0") dry-run-publish all
  $(basename "$0") publish all
  $(basename "$0") sdk-publish
EOF
}

case "$phase" in
  preflight)
    preflight
    ;;
  release-candidate)
    release_candidate
    ;;
  security)
    security
    ;;
  sdk-package)
    sdk_package
    ;;
  sdk-publish)
    sdk_publish
    ;;
  bump-version)
    bump_version "$@"
    ;;
  foundation)
    foundation
    ;;
  contracts)
    contracts
    ;;
  learning)
    learning
    ;;
  policy)
    policy
    ;;
  compat)
    compat
    ;;
  server)
    server
    ;;
  facade)
    facade
    ;;
  dry-run-publish)
    dry_run_publish "$@"
    ;;
  publish)
    publish "$@"
    ;;
  publish-plan)
    publish_plan
    ;;
  help|-h|--help)
    print_help
    ;;
  *)
    printf 'Unknown phase: %s\n\n' "$phase" >&2
    print_help >&2
    exit 1
    ;;
esac
