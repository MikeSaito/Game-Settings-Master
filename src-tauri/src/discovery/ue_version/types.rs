use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UeEngineFamily {
    Ue4,
    Ue5,
    Unknown,
}

impl UeEngineFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ue4 => "ue4",
            Self::Ue5 => "ue5",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "ue4" | "4" => Self::Ue4,
            "ue5" | "5" => Self::Ue5,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UeVersionInfo {
    pub family: UeEngineFamily,
    pub version: Option<String>,
}

pub(crate) fn resolve_family_score(score_ue4: i32, score_ue5: i32) -> UeEngineFamily {
    if score_ue5 > score_ue4 && score_ue5 >= 2 {
        UeEngineFamily::Ue5
    } else if score_ue4 > score_ue5 && score_ue4 >= 2 {
        UeEngineFamily::Ue4
    } else if score_ue5 >= 1 && score_ue4 == 0 {
        UeEngineFamily::Ue5
    } else if score_ue4 >= 1 && score_ue5 == 0 {
        UeEngineFamily::Ue4
    } else {
        UeEngineFamily::Unknown
    }
}

pub(crate) fn format_version(major: u32, minor: u32, patch: u32) -> String {
    if patch == 0 {
        format!("{major}.{minor}")
    } else {
        format!("{major}.{minor}.{patch}")
    }
}
