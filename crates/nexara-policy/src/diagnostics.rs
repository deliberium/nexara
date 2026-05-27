use nexara_core::{PolicyDiagnostic, PolicyDiagnosticCode, PolicyDiagnosticSeverity};

pub fn warning(code: PolicyDiagnosticCode, message: impl Into<String>) -> PolicyDiagnostic {
    PolicyDiagnostic {
        severity: PolicyDiagnosticSeverity::Warning,
        code,
        message: message.into(),
    }
}

pub fn error(code: PolicyDiagnosticCode, message: impl Into<String>) -> PolicyDiagnostic {
    PolicyDiagnostic {
        severity: PolicyDiagnosticSeverity::Error,
        code,
        message: message.into(),
    }
}
