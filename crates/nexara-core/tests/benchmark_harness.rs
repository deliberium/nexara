use nexara_core::{ToolBroker, ToolDescriptor, ToolRegistry, ToolSelectionRequest};

#[test]
#[ignore = "benchmark harness; run explicitly with --ignored"]
fn broker_selection_benchmark_harness() {
    let mut registry = ToolRegistry::new();
    for index in 0..1_000 {
        let mut tool = ToolDescriptor::read_only(format!("search_{index}"), "Search fixture");
        tool.tags = vec!["search".to_string(), format!("tenant_{}", index % 10)];
        registry.register(tool);
    }
    let broker = ToolBroker::new();
    let started = std::time::Instant::now();
    let selected = broker.select(
        &registry,
        &ToolSelectionRequest {
            prompt: "search tenant_3 records".to_string(),
            scopes: vec!["read".to_string()],
            max_tools: 8,
        },
    );
    assert_eq!(selected.len(), 8);
    eprintln!("broker selected 8 of 1000 tools in {:?}", started.elapsed());
}
