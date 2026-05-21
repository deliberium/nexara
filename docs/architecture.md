# Architecture

Nexara separates capability policy from host execution. A host owns
identity, context, final tool behavior, secrets, and deployment. Nexara owns the
reusable capability contract: descriptors, registries, brokered model exposure,
policy enforcement, remote discovery, calls, audit, and optional learned usage
signals.

![Nexara architecture diagram](assets/nexara-architecture.svg)

## Component Responsibilities

`nexara-core` defines the stable language of tools: descriptors, scopes, action
classes, trust tiers, broker requests, call requests, call results, registry
lookups, validation helpers, audit records, and shared errors.

`nexara-registry` owns signed skill indexes, product-neutral manifests,
`host_requirements`, signature verification, manifest hash validation, and
installed-skill storage contracts.

`nexara-remote` owns remote endpoint modeling, auth providers, allowlist checks,
diagnostics, and HTTP list/call behavior. Hosts still decide which endpoints are
trusted and whether imported endpoints are enabled.

`nexara-secrets` owns redacted secret values and sync/async store contracts.
Values are resolved by tools or host executors, not copied into descriptors,
prompts, learned records, or audit payloads.

`nexara-runtime` is the enforcement point. It combines the catalog, broker,
trust resolver, host executor, payload limits, confirmation policy, concurrency
limits, and audit sink.

`nexara-learning` is optional and bounded. It can adjust ranking, but it cannot
override hard policy decisions.

`nexara-server` provides an authenticated HTTP surface over runtime and registry
contracts for hosts that want a service boundary.

`nexara-compat-qwen` imports Qwen-style MCP configuration into Nexara endpoint
or registry records without enabling those tools outside host policy.

## Request Flow

1. The host builds a runtime with a catalog, executor, trust resolver, optional
   secret store, optional usage-signal provider, and audit sink.
2. The model or server asks for visible tools with a prompt and scope set.
3. The broker filters and ranks enabled descriptors.
4. A tool call arrives as a typed `ToolCallRequest`.
5. Runtime checks payload size, action class, scope, trust tier, confirmation,
   concurrency, and host allowlist where relevant.
6. Runtime delegates execution to the host executor or remote executor.
7. Runtime emits an audit record with outcome, duration, and result hash.
