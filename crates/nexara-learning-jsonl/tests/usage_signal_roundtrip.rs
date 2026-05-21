use nexara_learning::{
    ToolSelectionContext, ToolUsageEvent, ToolUsageEventKind, ToolUsageSignalStore,
};
use nexara_learning_jsonl::JsonlUsageSignalStore;

#[test]
fn jsonl_store_roundtrips_and_ignores_malformed_records() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("signals.jsonl");
    std::fs::write(&path, "{bad json}\n").unwrap();
    let store = JsonlUsageSignalStore::new(&path);
    store
        .record(ToolUsageEvent {
            tool_name: "search".to_string(),
            context: ToolSelectionContext::default(),
            kind: ToolUsageEventKind::Success,
            timestamp_unix_ms: 1,
            latency_ms: None,
        })
        .unwrap();

    assert_eq!(store.events_for_tool("search").unwrap().len(), 1);
}
