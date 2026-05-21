use nexara_learning::{
    ToolSelectionContext, ToolUsageEvent, ToolUsageEventKind, ToolUsageSignalStore,
};
use nexara_learning_memory::MemoryUsageSignalStore;

#[test]
fn memory_store_roundtrips() {
    let store = MemoryUsageSignalStore::default();
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
