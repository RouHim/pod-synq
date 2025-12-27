use crate::{
    error::{AppError, AppResult},
    models::SettingRequest,
    repository::{SettingKey, SettingRepository},
};

#[derive(Clone)]
pub struct SettingService {
    setting_repo: SettingRepository,
}

impl SettingService {
    pub fn new(setting_repo: SettingRepository) -> Self {
        Self { setting_repo }
    }

    pub async fn get_settings(
        &self,
        user_id: i64,
        scope: &str,
        podcast_url: Option<&str>,
        device_id: Option<i64>,
        episode_url: Option<&str>,
    ) -> AppResult<serde_json::Map<String, serde_json::Value>> {
        let settings = self
            .setting_repo
            .get_settings(user_id, scope, podcast_url, device_id, episode_url)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let mut result = serde_json::Map::new();
        for setting in settings {
            let value: serde_json::Value = serde_json::from_str(&setting.value)
                .unwrap_or_else(|_| serde_json::Value::String(setting.value.clone()));
            result.insert(setting.key, value);
        }

        Ok(result)
    }

    pub async fn save_settings(
        &self,
        user_id: i64,
        scope: &str,
        podcast_url: Option<&str>,
        device_id: Option<i64>,
        episode_url: Option<&str>,
        req: SettingRequest,
    ) -> AppResult<serde_json::Map<String, serde_json::Value>> {
        if let Some(to_set) = req.set {
            for (key, value) in to_set {
                let value_str = serde_json::to_string(&value).unwrap_or_default();
                let setting_key = SettingKey {
                    user_id,
                    scope,
                    podcast_url,
                    device_id,
                    episode_url,
                    key: &key,
                };
                self.setting_repo
                    .upsert_setting(setting_key, &value_str)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
        }

        if let Some(to_remove) = req.remove {
            for key in to_remove {
                let setting_key = SettingKey {
                    user_id,
                    scope,
                    podcast_url,
                    device_id,
                    episode_url,
                    key: &key,
                };
                self.setting_repo
                    .delete_setting(setting_key)
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
        }

        self.get_settings(user_id, scope, podcast_url, device_id, episode_url)
            .await
    }
}
