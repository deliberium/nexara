use nexara_core::{ToolDescriptor, ToolSelectionRequest, ToolUsageSignalProvider};
use nexara_learning::{
    InProcessUsageSignalStore, LearningScorer, LearningSignalProvider, ToolSelectionContext,
    ToolUsageEvent, ToolUsageEventKind, ToolUsageSignalStore,
};

fn main() -> anyhow::Result<()> {
    let store = InProcessUsageSignalStore::default();
    let context = ToolSelectionContext::default();
    store.record(ToolUsageEvent {
        tool_name: "search".to_string(),
        context,
        kind: ToolUsageEventKind::Success,
        timestamp_unix_ms: 1_800_000_000_000,
        latency_ms: Some(42),
    })?;

    let provider = LearningSignalProvider::new(store, LearningScorer::default());
    let tool = ToolDescriptor::read_only("search", "Search an example corpus");
    let request = ToolSelectionRequest {
        prompt: "search the corpus".to_string(),
        scopes: vec!["read".to_string()],
        max_tools: 4,
    };
    println!("adjustment: {}", provider.score_adjustment(&tool, &request));
    println!(
        "{}",
        serde_json::to_string_pretty(&provider.explain_tool("search")?)?
    );
    Ok(())
}
