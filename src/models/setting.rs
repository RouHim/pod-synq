use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Setting {
    pub id: i64,
    pub user_id: i64,
    pub scope: String,
    pub podcast_url: Option<String>,
    pub device_id: Option<i64>,
    pub episode_url: Option<String>,
    pub key: String,
    pub value: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SettingRequest {
    pub set: Option<serde_json::Map<String, serde_json::Value>>,
    pub remove: Option<Vec<String>>,
}

impl<'de> Deserialize<'de> for SettingRequest {
    /// Accepts both the gPodder wire format `{"set": {...}, "remove": [...]}`
    /// and a flat key-value object treated as `set`.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;
        let obj = value
            .as_object()
            .ok_or_else(|| D::Error::custom("settings payload must be an object"))?;

        if let Some(set) = obj.get("set") {
            return Ok(SettingRequest {
                set: set.as_object().cloned(),
                remove: obj.get("remove").and_then(|r| {
                    serde_json::from_value(r.clone())
                        .map_err(|e| D::Error::custom(format!("invalid remove list: {e}")))
                        .ok()
                }),
            });
        }

        match obj.get("remove") {
            Some(remove) => Ok(SettingRequest {
                set: None,
                remove: serde_json::from_value(remove.clone())
                    .map_err(|e| D::Error::custom(format!("invalid remove list: {e}")))?,
            }),
            None => Ok(SettingRequest {
                set: Some(obj.clone()),
                remove: None,
            }),
        }
    }
}
