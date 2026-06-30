use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WslInstance {
    pub name: String,
    pub state: InstanceState,
    pub version: u8,
    pub default: bool,
    pub distribution: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WslVersion {
    pub wsl_version: Option<String>,
    pub kernel_version: Option<String>,
    pub windows_version: Option<String>,
    pub fields: HashMap<String, String>,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum InstanceState {
    Running,
    Stopped,
    Unknown,
}

impl std::str::FromStr for InstanceState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Running" => Ok(InstanceState::Running),
            "Stopped" => Ok(InstanceState::Stopped),
            _ => Ok(InstanceState::Unknown),
        }
    }
}
