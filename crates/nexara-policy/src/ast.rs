use nexara_core::PolicyEffect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPolicy {
    pub clauses: Vec<PolicyClause>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyClause {
    pub effect: PolicyEffect,
    pub resource: String,
    pub operation: String,
    pub reason: String,
}

impl PolicyClause {
    pub fn capability(&self) -> String {
        format!("{}.{}", self.resource, self.operation)
    }
}
