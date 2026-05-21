use crate::error::{NexaraError, NexaraResult};
use crate::tool::ToolDescriptor;

pub fn validate_tool_descriptor(descriptor: &ToolDescriptor) -> NexaraResult<()> {
    validate_tool_name(&descriptor.name)?;
    if descriptor.description.trim().is_empty() {
        return Err(NexaraError::InvalidDescriptor(
            "description must not be empty".to_string(),
        ));
    }
    for scope in &descriptor.scopes {
        validate_scope_name(scope)?;
    }
    Ok(())
}

pub fn validate_tool_name(name: &str) -> NexaraResult<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(NexaraError::InvalidDescriptor(
            "tool name must not be empty".to_string(),
        ));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(NexaraError::InvalidDescriptor(format!(
            "invalid tool name '{name}'"
        )));
    }
    Ok(())
}

pub fn validate_scope_name(scope: &str) -> NexaraResult<()> {
    let trimmed = scope.trim();
    if trimmed.is_empty() {
        return Err(NexaraError::InvalidDescriptor(
            "scope must not be empty".to_string(),
        ));
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == ':' || ch == '.')
    {
        return Err(NexaraError::InvalidDescriptor(format!(
            "invalid scope '{scope}'"
        )));
    }
    Ok(())
}
