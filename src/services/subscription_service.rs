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
        Ok(subs)
    }

    pub async fn get_all_subscriptions(&self, user_id: i64) -> AppResult<Vec<String>> {
        let subs = self
            .sub_repo
            .list_all_urls_by_user(user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(subs)
    }

    pub async fn get_changes_since(
        &self,
        user_id: i64,
        device_id: i64,
        since: i64,
    ) -> AppResult<(Vec<String>, Vec<String>)> {
        self.sub_repo
            .get_changes_since(user_id, device_id, since)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn set_subscriptions(
        &self,
        user_id: i64,
        device_id: i64,
        podcast_urls: Vec<String>,
    ) -> AppResult<()> {
        self.sub_repo
            .set_subscriptions(user_id, device_id, podcast_urls)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn upload_changes(
        &self,
        user_id: i64,
        device_id: i64,
        changes: SubscriptionChanges,
    ) -> AppResult<()> {
        let count = changes.add.len() + changes.remove.len();
        self.sub_repo
            .apply_changes(user_id, device_id, changes)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!(
            "Uploaded {} subscription changes for device {}",
            count,
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
            .map_err(|e| AppError::Internal(e.to_string()))
    }
}
