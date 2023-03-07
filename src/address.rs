use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub enum PolicyId {
    Lovelace,
    NativeToken(String, Option<String>),
}

impl PolicyId {
    pub fn ada() -> PolicyId {
        PolicyId::Lovelace
    }

    pub fn native_token(id: &str, asset: &Option<String>) -> PolicyId {
        PolicyId::NativeToken(id.to_string(), asset.to_owned())
    }

    pub fn to_str(&self) -> Option<String> {
        match self {
            PolicyId::Lovelace => None,
            PolicyId::NativeToken(id, maybe_asset) => {
                if let Some(asset) = maybe_asset {
                    Some(format!("{id}-{asset}"))
                } else {
                    Some(id.to_string())
                }
            }
        }
    }

    pub fn asset_name(&self) -> Option<String> {
        match self {
            PolicyId::Lovelace => None,
            PolicyId::NativeToken(_, asset_name) => asset_name.to_owned(),
        }
    }
}
