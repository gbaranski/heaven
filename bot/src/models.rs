use serenity::model::prelude::UserId;
use serde::Serialize;
use serde::Deserialize;

pub type AngelID = uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct Angel {
    pub id: AngelID,
    pub discord_id: UserId,
    pub discord_name: String,
    pub minecraft_name: String,
    pub minecraft_type: MinecraftType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
pub enum MinecraftType {
    Premium,
    Cracked,
}

impl std::fmt::Display for MinecraftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Premium => "premium",
            Self::Cracked => "cracked",
        };
        f.write_str(name)
    }
}


impl std::str::FromStr for MinecraftType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s {
            "premium" => Self::Premium,
            "cracked" => Self::Cracked,
            other => {
                return Err(format!("unknown minecraft type of {other}"));
            }
        };
        Ok(v)
    }
}