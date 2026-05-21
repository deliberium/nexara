use nexara_learning::{
    InProcessUsageSignalStore, LearningScorer, ToolSelectionContext, ToolUsageEvent,
    ToolUsageEventKind, ToolUsageSignalStore,
};

#[test]
fn learned_adjustments_are_bounded() {
    let store = InProcessUsageSignalStore::default();
    for idx in 0..100 {
        store
            .record(ToolUsageEvent {
                tool_name: "search".to_string(),
                context: ToolSelectionContext::default(),
                kind: ToolUsageEventKind::Success,
                timestamp_unix_ms: idx,
                latency_ms: None,
            })
            .unwrap();
    }

    let scorer = LearningScorer::new(1.5);
    let score = scorer.score_events(&store.events_for_tool("search").unwrap());
    assert_eq!(score, 1.5);
}
