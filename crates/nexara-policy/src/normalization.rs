pub fn normalize_policy_text(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .replace([';', '\n', '\t'], ",")
        .replace("can't", "cannot")
        .replace("can not", "cannot")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn normalize_resource(input: &str) -> String {
    let mut value = input
        .trim()
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '.' && ch != '-' && ch != '_')
        .to_ascii_lowercase()
        .replace(' ', "_");
    value = match value.as_str() {
        "refund" | "refunding" => "refunds".to_string(),
        "order" => "orders".to_string(),
        "customer" => "customers".to_string(),
        "doc" | "document" | "documents" => "docs".to_string(),
        "email" => "emails".to_string(),
        "file" => "files".to_string(),
        other => singular_to_pluralish(other),
    };
    value
}

pub fn normalize_operation(input: &str) -> Option<String> {
    match input.trim().to_ascii_lowercase().as_str() {
        "read" | "reads" | "view" | "views" | "inspect" | "lookup" | "lookups" | "get" | "gets"
        | "list" | "lists" => Some("read".to_string()),
        "search" | "searches" => Some("search".to_string()),
        "create" | "creates" | "add" | "adds" => Some("create".to_string()),
        "update" | "updates" | "edit" | "edits" => Some("update".to_string()),
        "delete" | "deletes" | "remove" | "removes" => Some("delete".to_string()),
        "send" | "sends" | "deliver" | "delivers" => Some("send".to_string()),
        "export" | "exports" | "download" | "downloads" => Some("export".to_string()),
        "cancel" | "cancels" | "void" | "voids" => Some("cancel".to_string()),
        "capture" | "captures" | "charge" | "charges" | "pay" | "pays" => {
            Some("capture".to_string())
        }
        "execute" | "executes" | "run" | "runs" => Some("execute".to_string()),
        "refund" | "refunds" | "refunding" => Some("create".to_string()),
        "draft" | "drafts" => Some("create".to_string()),
        _ => None,
    }
}

pub fn infer_resource_from_operation(operation_word: &str) -> Option<String> {
    match operation_word {
        "refund" | "refunds" | "refunding" => Some("refunds".to_string()),
        _ => None,
    }
}

fn singular_to_pluralish(value: &str) -> String {
    if value.ends_with('s') || value.contains('.') {
        value.to_string()
    } else {
        format!("{value}s")
    }
}
