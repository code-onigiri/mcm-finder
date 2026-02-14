// Enum definitions used across the application
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModLoader {
    Fabric,
    Forge,
    Quilt,
    NeoForge,
}

impl ModLoader {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "fabric" => Some(Self::Fabric),
            "forge" => Some(Self::Forge),
            "quilt" => Some(Self::Quilt),
            "neoforge" => Some(Self::NeoForge),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    Modrinth,
    CurseForge,
    #[allow(dead_code)]
    GitHub, // Future
    #[allow(dead_code)]
    McBaike, // Future (mc百科)
}

impl Provider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Modrinth => "modrinth",
            Self::CurseForge => "curseforge",
            Self::GitHub => "github",
            Self::McBaike => "mcbaike",
        }
    }
}
