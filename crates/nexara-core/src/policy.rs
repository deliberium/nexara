use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    Builtin,
    RemoteMcp,
    RemoteNexara,
    LocalExternalProcess,
    WasmComponent,
    Custom,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ActionClass {
    Read,
    Write,
    Execute,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationPolicy {
    Never,
    Always,
    OnWrite,
    OnFirstUse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum TrustProfile {
    Observe,
    Assist,
    ActWithConfirmation,
    FullOperator,
}

impl TrustProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Assist => "assist",
            Self::ActWithConfirmation => "act_with_confirmation",
            Self::FullOperator => "full_operator",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EffectiveTrustPolicy {
    pub profile: TrustProfile,
    pub allow_read: bool,
    pub allow_write: bool,
    pub allow_execute: bool,
    pub require_confirmation_for_write: bool,
    pub require_confirmation_for_execute: bool,
}

impl EffectiveTrustPolicy {
    pub fn for_profile(profile: TrustProfile) -> Self {
        match profile {
            TrustProfile::Observe => Self {
                profile,
                allow_read: true,
                allow_write: false,
                allow_execute: false,
                require_confirmation_for_write: true,
                require_confirmation_for_execute: true,
            },
            TrustProfile::Assist => Self {
                profile,
                allow_read: true,
                allow_write: false,
                allow_execute: false,
                require_confirmation_for_write: true,
                require_confirmation_for_execute: true,
            },
            TrustProfile::ActWithConfirmation => Self {
                profile,
                allow_read: true,
                allow_write: true,
                allow_execute: true,
                require_confirmation_for_write: true,
                require_confirmation_for_execute: true,
            },
            TrustProfile::FullOperator => Self {
                profile,
                allow_read: true,
                allow_write: true,
                allow_execute: true,
                require_confirmation_for_write: false,
                require_confirmation_for_execute: false,
            },
        }
    }
}
