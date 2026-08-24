use async_trait::async_trait;

use crate::error::AppResult;
use crate::models::{
    Device, EpisodeAction, EpisodeActionQuery, FavoriteEpisode, FavoriteMetadata, Podcast, Session,
    Setting, SubscriptionChanges, User,
};

use super::episode_action_repository::EpisodeActionWithDevice;
use super::setting_repository::SettingKey;

/// Trait for user repository operations
#[async_trait]
pub trait UserRepositoryTrait: Send + Sync {
    async fn create(&self, username: &str, password_hash: &str, is_admin: bool) -> AppResult<User>;
    async fn find_by_id(&self, id: i64) -> AppResult<Option<User>>;
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    async fn is_empty(&self) -> AppResult<bool>;
}

/// Trait for device repository operations
#[async_trait]
pub trait DeviceRepositoryTrait: Send + Sync {
    async fn create(
        &self,
        user_id: i64,
        device_id: &str,
        caption: Option<&str>,
        device_type: Option<&str>,
    ) -> AppResult<i64>;
    async fn find_by_device_id(&self, user_id: i64, device_id: &str) -> AppResult<Option<Device>>;
    async fn list_by_user(&self, user_id: i64) -> AppResult<Vec<Device>>;
}

/// Trait for session repository operations
#[async_trait]
pub trait SessionRepositoryTrait: Send + Sync {
    async fn create(&self, id: &str, user_id: i64, expires_at: i64) -> AppResult<()>;
    async fn find_by_id(&self, id: &str) -> AppResult<Option<Session>>;
    async fn delete(&self, id: &str) -> AppResult<()>;
}

/// Trait for subscription repository operations
#[async_trait]
pub trait SubscriptionRepositoryTrait: Send + Sync {
    async fn list_by_device(&self, user_id: i64, device_id: i64) -> AppResult<Vec<String>>;
    async fn list_all_urls_by_user(&self, user_id: i64) -> AppResult<Vec<String>>;
    async fn get_changes_since(
        &self,
        user_id: i64,
        device_id: i64,
        since: i64,
    ) -> AppResult<(Vec<String>, Vec<String>)>;
    async fn set_subscriptions(
        &self,
        user_id: i64,
        device_id: i64,
        podcast_urls: Vec<String>,
    ) -> AppResult<()>;
    async fn apply_changes(
        &self,
        user_id: i64,
        device_id: i64,
        changes: SubscriptionChanges,
    ) -> AppResult<()>;
    async fn count(&self, user_id: i64, device_id: Option<i64>) -> AppResult<i64>;
}

/// Trait for episode action repository operations
#[async_trait]
pub trait EpisodeActionRepositoryTrait: Send + Sync {
    async fn list(
        &self,
        user_id: i64,
        query: EpisodeActionQuery,
    ) -> AppResult<Vec<EpisodeActionWithDevice>>;
    async fn upload(&self, actions: Vec<EpisodeAction>) -> AppResult<()>;
}

/// Trait for setting repository operations
#[async_trait]
pub trait SettingRepositoryTrait: Send + Sync {
    async fn get_settings(
        &self,
        user_id: i64,
        scope: &str,
        podcast_url: Option<&str>,
        device_id: Option<i64>,
        episode_url: Option<&str>,
    ) -> AppResult<Vec<Setting>>;
    async fn upsert_setting(&self, key: SettingKey<'_>, value: &str) -> AppResult<()>;
    async fn delete_setting(&self, key: SettingKey<'_>) -> AppResult<()>;
}

/// Trait for favorite repository operations
#[async_trait]
pub trait FavoriteRepositoryTrait: Send + Sync {
    async fn add_favorite(&self, user_id: i64, metadata: &FavoriteMetadata<'_>) -> AppResult<i64>;
    async fn remove_favorite(&self, user_id: i64, episode_url: &str) -> AppResult<()>;
    async fn get_user_favorites(&self, user_id: i64) -> AppResult<Vec<FavoriteEpisode>>;
}

/// Trait for device sync repository operations
#[async_trait]
pub trait DeviceSyncRepositoryTrait: Send + Sync {
    async fn create_group(&self, user_id: i64) -> AppResult<i64>;
    async fn add_device_to_group(&self, group_id: i64, device_id: i64) -> AppResult<()>;
    async fn remove_device_from_group(&self, device_id: i64) -> AppResult<()>;
    async fn get_device_group(&self, device_id: i64) -> AppResult<Option<i64>>;
    async fn get_group_devices(&self, group_id: i64) -> AppResult<Vec<i64>>;
    async fn get_user_groups(&self, user_id: i64) -> AppResult<Vec<i64>>;
    async fn delete_group(&self, group_id: i64) -> AppResult<()>;
    async fn merge_groups(&self, target_group_id: i64, source_group_id: i64) -> AppResult<()>;
}

/// Trait for podcast repository operations
#[async_trait]
pub trait PodcastRepositoryTrait: Send + Sync {
    async fn get_by_urls(&self, urls: &[String]) -> AppResult<Vec<Podcast>>;
}
