use crate::error::AppResult;
use crate::models::Device;
use crate::repository::traits::DeviceRepositoryTrait;
use crate::repository::DeviceRepository;

#[derive(Clone)]
pub struct DeviceService {
    device_repo: DeviceRepository,
}

impl DeviceService {
    pub fn new(device_repo: DeviceRepository) -> Self {
        Self { device_repo }
    }

    pub async fn find_by_device_id(&self, user_id: i64, device_id: &str) -> AppResult<Device> {
        self.device_repo
            .find_by_device_id(user_id, device_id)
            .await?
            .ok_or_else(|| crate::error::AppError::DeviceNotFound(device_id.to_string()))
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

        let device_id = self
            .device_repo
            .create(user_id, device_id, Some(caption), device_type)
            .await?;

        Ok(device_id)
    }

    pub async fn list_user_devices(&self, user_id: i64) -> AppResult<Vec<Device>> {
        self.device_repo.list_by_user(user_id).await
    }
}
