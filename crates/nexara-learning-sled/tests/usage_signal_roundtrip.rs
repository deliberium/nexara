use nexara_learning::{
    ToolSelectionContext, ToolUsageEvent, ToolUsageEventKind, ToolUsageSignalStore,
};
use nexara_learning_sled::SledUsageSignalStore;

#[test]
fn sled_store_roundtrips() {
    let dir = tempfile::tempdir().unwrap();
    let store = SledUsageSignalStore::open(dir.path()).unwrap();
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
