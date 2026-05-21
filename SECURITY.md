# Security Policy

Nexara is a capability and tool runtime. Security bugs can expose tools to the
wrong caller, call remote services unexpectedly, leak secrets through prompts or
logs, or weaken signed registry guarantees. Please report those issues
privately before public disclosure.

## Reporting A Vulnerability

Email the Nexara maintainers using the private security contact for
the repository once it is published. Until that address is public, report
through the Nexara maintainer channel and mark the report as confidential.

Please include:

- affected crate and version or commit
- minimal reproduction steps
- expected and actual behavior
- whether secrets, remote calls, registry verification, or auth are involved
- any suspected downstream host impact

We will acknowledge reports as soon as practical, triage severity, coordinate a
fix, and credit reporters when requested.

## What To Report

Report issues involving:

- unsigned or tampered skill indexes being accepted
- manifest hash verification bypass
- `host_requirements` being ignored in a way that allows incompatible installs
- tool descriptors leaking secret values or unbounded prompt content
- runtime policy bypass for action class, scope, trust tier, confirmation, or host allowlists
- server auth bypass or admin API exposure
- secret-store redaction failures
- learned usage signals overriding hard policy
- audit records containing raw secrets or large sensitive payloads

## Supported Security Posture

Nexara assumes:

- signed registries are verified before install
- remote hosts are explicitly allowlisted by the host
- secrets never appear in descriptors, prompts, learned-signal records, or audit payloads
- write and execute tools require explicit policy support and confirmation where configured
- admin APIs require authentication by default
- host applications own user identity, environment parsing, and final execution authority

## Hardening Guidance

- Use bearer auth or a host authorizer for all server routes outside local demos.
- Keep development no-auth mode disabled in production.
- Use a host-provided async secret store for KMS, vault, browser, or OS keychain integrations.
- Keep registry trust bundles small and review key rotation procedures.
- Treat Qwen or MCP config imports as candidate endpoints; do not auto-enable them without host policy review.
- Partition learned usage signals by host, user, workspace, or surface when the deployment is multi-tenant.

## Disclosure Expectations

Please do not publish exploit details until a fix is available and maintainers
have had a reasonable chance to coordinate downstream host updates.
