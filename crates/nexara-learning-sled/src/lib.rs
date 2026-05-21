use nexara_learning::{ToolUsageEvent, ToolUsageSignalStore};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SledUsageSignalStore {
    db: sled::Db,
}

impl SledUsageSignalStore {
    pub fn open(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}

impl ToolUsageSignalStore for SledUsageSignalStore {
    fn record(&self, event: ToolUsageEvent) -> anyhow::Result<()> {
        let key = format!(
            "tool:{}:{}:{}",
            event.tool_name,
            event.timestamp_unix_ms,
            Uuid::new_v4()
        );
        self.db
            .insert(key.as_bytes(), serde_json::to_vec(&event)?)?;
        self.db.flush()?;
        Ok(())
    }

    fn events_for_tool(&self, tool_name: &str) -> anyhow::Result<Vec<ToolUsageEvent>> {
        let prefix = format!("tool:{tool_name}:");
        let mut events = Vec::new();
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value) = item?;
            events.push(serde_json::from_slice(&value)?);
        }
        Ok(events)
    }
}
