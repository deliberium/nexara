# Skill Manifest Specification

Nexara manifests use `host_requirements`.

```toml
[host_requirements]
nexara = ">=0.1.0"
example_app = ">=0.2.0"
```

Legacy app-specific compatibility fields are not part of Nexara.

## Semantic Capabilities

Manifests can describe semantic capabilities so policy contracts can reason
about resource operations instead of only server or tool names.

```toml
[[capabilities]]
id = "orders.refunds.create"
name = "Create order refund"
description = "Issue a refund for an order"
display_name = "Create order refund"
resource = "orders"
resource_path = ["orders", "refunds"]
operation = "create"
action_class = "write"
sensitivity = "financial"
aliases = ["refund order", "issue refund"]

[[effects]]
resource = "refunds"
operation = "create"
writes = true
external_side_effect = true
financial_side_effect = true
irreversible = false
```

When a runtime policy contract is active, missing semantic capability metadata is
a denial condition.
