use crate::error::{NexaraError, NexaraResult};
use crate::tool::{ToolCapability, ToolDescriptor};

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
    for capability in &descriptor.capabilities {
        validate_tool_capability(capability)?;
    }
    Ok(())
}

pub fn validate_tool_capability(capability: &ToolCapability) -> NexaraResult<()> {
    validate_capability_id(&capability.id)?;
    if capability.resource.trim().is_empty() {
        return Err(NexaraError::InvalidDescriptor(
            "capability resource must not be empty".to_string(),
        ));
    }
    if capability.operation.trim().is_empty() {
        return Err(NexaraError::InvalidDescriptor(
            "capability operation must not be empty".to_string(),
        ));
    }
    let mut segments = capability.id.split('.').collect::<Vec<_>>();
    if segments.len() < 2 {
        return Err(NexaraError::InvalidDescriptor(format!(
            "capability id '{}' must include resource and operation",
            capability.id
        )));
    }
    let operation = segments.pop().unwrap_or_default();
    if operation != capability.operation {
        return Err(NexaraError::InvalidDescriptor(format!(
            "capability id '{}' operation does not match '{}'",
            capability.id, capability.operation
        )));
    }
    if capability.resource_path.is_empty() {
        return Err(NexaraError::InvalidDescriptor(format!(
            "capability '{}' resource_path must not be empty",
            capability.id
        )));
    }
    if capability.resource_path.join(".") != segments.join(".") {
        return Err(NexaraError::InvalidDescriptor(format!(
            "capability '{}' resource_path must match id resource path",
            capability.id
        )));
    }
    if segments.len() > 1
        && (capability
            .display_name
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
            || capability
                .description
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty())
    {
        return Err(NexaraError::InvalidDescriptor(format!(
            "capability '{}' requires display_name and description for deeper resource paths",
            capability.id
        )));
    }
    Ok(())
}

pub fn validate_capability_id(id: &str) -> NexaraResult<()> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return Err(NexaraError::InvalidDescriptor(
            "capability id must not be empty".to_string(),
        ));
    }
    if !trimmed.chars().all(|ch| {
        ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' || ch == '-' || ch == '.'
    }) {
        return Err(NexaraError::InvalidDescriptor(format!(
            "invalid capability id '{id}'"
        )));
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
