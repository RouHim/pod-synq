use crate::{
    error::{AppError, AppResult},
    repository::DeviceRepository,
};

#[derive(Clone)]
pub struct DeviceService {
    device_repo: DeviceRepository,
}

impl DeviceService {
    pub fn new(device_repo: DeviceRepository) -> Self {
        Self { device_repo }
    }

    pub async fn get_or_create_device(
        &self,
        user_id: i64,
        device_id: &str,
        caption: Option<&str>,
        device_type: Option<&str>,
    ) -> AppResult<i64> {
        if let Some(device) = self
            .device_repo
            .find_by_device_id(user_id, device_id)
            .await?
        {
            tracing::debug!("Device found: {}", device_id);
            return Ok(device.id);
        }

        let caption = caption.unwrap_or("Unknown Device");
        tracing::info!("Creating device: {} for user {}", device_id, user_id);

        let device = self
            .device_repo
            .create(user_id, device_id, Some(caption), device_type)
            .await?;

        Ok(device.id)
    }

    pub async fn get_device(&self, id: i64) -> AppResult<crate::models::Device> {
        self.device_repo
            .find_by_id(id)
            .await?
            .ok_or(AppError::DeviceNotFound)
    }

    pub async fn list_user_devices(&self, user_id: i64) -> AppResult<Vec<crate::models::Device>> {
        self.device_repo
            .list_by_user(user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn update_device(
        &self,
        id: i64,
        caption: Option<&str>,
        device_type: Option<&str>,
    ) -> AppResult<crate::models::Device> {
        self.device_repo
            .update(id, caption, device_type)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn delete_device(&self, id: i64) -> AppResult<()> {
        self.device_repo.delete(id).await?;
        tracing::info!("Deleted device ID: {}", id);
        Ok(())
    }

    pub async fn count_devices(&self, user_id: Option<i64>) -> AppResult<i64> {
        self.device_repo.count(user_id).await
    }
}
