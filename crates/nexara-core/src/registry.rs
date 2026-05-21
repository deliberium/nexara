use crate::tool::ToolDescriptor;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, ToolDescriptor>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, tool: ToolDescriptor) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&ToolDescriptor> {
        self.tools.get(name)
    }

    pub fn all(&self) -> Vec<ToolDescriptor> {
        let mut tools: Vec<_> = self.tools.values().cloned().collect();
        tools.sort_by(|left, right| left.name.cmp(&right.name));
        tools
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}
