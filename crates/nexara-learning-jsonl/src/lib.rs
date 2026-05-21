use nexara_learning::{ToolUsageEvent, ToolUsageSignalStore};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct JsonlUsageSignalStore {
    path: PathBuf,
}

impl JsonlUsageSignalStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl ToolUsageSignalStore for JsonlUsageSignalStore {
    fn record(&self, event: ToolUsageEvent) -> anyhow::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", serde_json::to_string(&event)?)?;
        Ok(())
    }

    fn events_for_tool(&self, tool_name: &str) -> anyhow::Result<Vec<ToolUsageEvent>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.path)?;
        let mut events = Vec::new();
        for line in std::io::BufReader::new(file).lines() {
            let Ok(line) = line else {
                continue;
            };
            let Ok(event) = serde_json::from_str::<ToolUsageEvent>(&line) else {
                continue;
            };
            if event.tool_name == tool_name {
                events.push(event);
            }
        }
        Ok(events)
    }
}
