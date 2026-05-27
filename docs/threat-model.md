# Threat Model

Nexara does not trust model output. Tool calls are policy checked before host
execution, and learned usage signals cannot override hard policy.

## Policy Authoring Risks

- Ambiguous natural-language policy can produce a broader rule than intended.
- Broad allow rules can make an agent useful but unsafe.
- Missing semantic capability metadata can make policy claims impossible to
  enforce.
- A malicious or sloppy tool publisher can mislabel side effects.
- Unsigned policy contracts can drift between review and deployment.

## Mitigations

- `nexara-policy` is deterministic and emits diagnostics for ambiguity,
  conflicts, and overbroad language.
- Deny overrides allow.
- When a policy contract is active, tools without semantic capability metadata
  are denied.
- Signed skill registries protect capability metadata during distribution.
- Signed policy contracts support enterprise review and deployment workflows.
- Runtime audit includes matched rule IDs and explanations.
