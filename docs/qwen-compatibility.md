# Qwen Compatibility

`nexara-compat-qwen` imports Qwen-family MCP configuration into Nexara remote
endpoint records. Import is intentionally not enablement: every imported server
is treated as a candidate until a host approval store marks it approved and the
runtime policy permits discovery/calls.

## Mapping

Qwen-style configuration:

```json
{
  "mcpServers": {
    "github": {
      "url": "https://tools.example.com/mcp"
    }
  }
}
```

maps to:

- `QwenImportRecord.name = "github"`
- `QwenImportRecord.endpoint.base_url = "https://tools.example.com/mcp"`
- `QwenImportRecord.approved = false` unless a `QwenApprovalStore` says otherwise

Command/process transports are rejected by Nexara; hosts that want local process
execution need an explicit sandboxing design outside Qwen import.

## Approval Storage

Hosts can provide a `QwenApprovalStore` implementation backed by local files,
databases, enterprise policy, or review workflows. The crate also provides
`MemoryQwenApprovalStore` for tests and demos.

Approval only records host review state. It does not bypass Nexara runtime
checks for trust tier, scope, action class, confirmation, payload limits, remote
host allowlists, or audit.
