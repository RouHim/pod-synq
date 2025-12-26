use crate::{
    error::{AppError, AppResult},
    models::SubscriptionChanges,
    repository::SubscriptionRepository,
};

#[derive(Clone)]
pub struct SubscriptionService {
    sub_repo: SubscriptionRepository,
}

impl SubscriptionService {
    pub fn new(sub_repo: SubscriptionRepository) -> Self {
        Self { sub_repo }
    }

    pub async fn get_subscriptions(&self, user_id: i64, device_id: i64) -> AppResult<Vec<String>> {
        let subs = self
            .sub_repo
            .list_by_device(user_id, device_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        subs.iter().map(|s| s.podcast_url.clone()).collect()
    }

    pub async fn add_subscription(
        &self,
        user_id: i64,
        device_id: i64,
        podcast_url: &str,
    ) -> AppResult<()> {
        self.sub_repo
            .add(user_id, device_id, podcast_url)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!(
            "Added subscription: {} for device {}",
            podcast_url,
            device_id,
        );
        Ok(())
    }

    pub async fn remove_subscription(
        &self,
        user_id: i64,
        device_id: i64,
        podcast_url: &str,
    ) -> AppResult<()> {
        self.sub_repo
            .remove(user_id, device_id, podcast_url)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!(
            "Removed subscription: {} for device {}",
            podcast_url,
            device_id,
        );
        Ok(())
    }

    pub async fn upload_changes(
        &self,
        user_id: i64,
        device_id: i64,
        changes: SubscriptionChanges,
    ) -> AppResult<()> {
        self.sub_repo
            .upload(user_id, device_id, changes)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!(
            "Uploaded {} subscription changes for device {}",
            changes.add.len() + changes.remove.len(),
            device_id,
        );
        Ok(())
    }

    pub async fn count_subscriptions(
        &self,
        user_id: i64,
        device_id: Option<i64>,
    ) -> AppResult<i64> {
        self.sub_repo
            .count(user_id, device_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }
}
