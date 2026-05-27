# nexara-policy

Deterministic one-line policy authoring for Nexara: write a compact intent,
inspect the generated contract, simulate it against a catalog, then enforce the
same policy at runtime.

```rust
use nexara_policy::{PolicyCompileOptions, PolicyCompiler};

let compiled = PolicyCompiler::default()
    .compile("reads orders, never refunds", PolicyCompileOptions::default())?;
println!("{}", compiled.explain_markdown());
# Ok::<(), anyhow::Error>(())
```

This crate is opt-in from the `nexara` facade via the `policy` feature.

The compiler is intentionally deterministic. It supports constrained grammar,
host-provided aliases, stable JSON output, catalog simulation, Markdown
explanations, and signed policy contracts for review workflows.
