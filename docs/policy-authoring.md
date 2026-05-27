# Policy Authoring

Nexara policy authoring turns a compact human sentence into an inspectable
runtime contract.

```text
reads orders, never refunds
```

The compiler expands that intent into explicit rules over semantic capabilities:

- allow `orders.read`
- allow `orders.search`
- require confirmation for `orders.export`
- deny `refunds.create`
- deny `orders.refunds.create`

## Mental Model

Policy authoring has three steps:

1. Compile plain intent into a `PolicyContract`.
2. Simulate the contract against a catalog before exposing tools to a model.
3. Enforce the same contract at runtime and audit the matched rule.

When a policy contract is active, tools without semantic capability metadata are
denied. This is deliberate: a sentence like `reads orders, never refunds` is only
meaningful if every candidate tool can prove what capability it represents.

## Capability Names

Capability IDs are product-neutral dot-delimited paths. The final segment is the
operation; preceding segments are the resource path.

```text
orders.read
orders.refunds.create
customers.notes.read
```

Keep vendor names in tool names or publisher metadata, not capability IDs.

Hosts can pass a small alias dictionary into `PolicyCompileOptions` when their
domain has preferred terms, for example mapping `returns` to `refunds`. Aliases
are resolved before parsing so the emitted contract remains product-neutral.

## CLI

```bash
nexara policy compile "reads orders, never refunds"
nexara policy compile "reads orders, never refunds" --json
nexara policy simulate contract.json --catalog examples/policy-shop/catalog.json
nexara policy sign contract.json --key-id security-review-2026 --key-file ./policy-review.key
nexara policy verify signed-contract.json --trusted-keys ./trusted-policy-keys.json
```

## Enterprise Review

Policy contracts can be signed after review. Production hosts can verify a signed
contract before activating it in `nexara-runtime`.

## Demo

```bash
cargo run -p policy-shop -- "reads orders, never refunds"
```

The demo shows an order lookup being allowed, order export requiring
confirmation, and refund creation being denied with an explanation.
