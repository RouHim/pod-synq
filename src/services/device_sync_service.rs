use crate::error::{AppError, AppResult};
use crate::models::DeviceSyncStatus;
use crate::repository::{DeviceRepository, DeviceSyncRepository};
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct DeviceSyncService {
    device_sync_repo: DeviceSyncRepository,
    device_repo: DeviceRepository,
}

impl DeviceSyncService {
    pub fn new(device_sync_repo: DeviceSyncRepository, device_repo: DeviceRepository) -> Self {
        Self {
            device_sync_repo,
            device_repo,
        }
    }

    /// Get current device synchronization status for a user
    pub async fn get_sync_status(&self, user_id: i64) -> AppResult<DeviceSyncStatus> {
        // Get all devices for this user
        let all_devices = self.device_repo.list_by_user(user_id).await?;
        let all_device_ids: Vec<String> = all_devices.iter().map(|d| d.device_id.clone()).collect();

        // Get all sync groups for this user
        let group_ids = self.device_sync_repo.get_user_groups(user_id).await?;

        let mut synchronized: Vec<Vec<String>> = Vec::new();
        let mut synced_device_ids: HashSet<String> = HashSet::new();

        // For each group, get the devices in it
        for group_id in group_ids {
            let device_db_ids = self.device_sync_repo.get_group_devices(group_id).await?;

            // Convert device database IDs to device_id strings
            let mut group_device_ids = Vec::new();
            for device_db_id in device_db_ids {
                if let Some(device) = all_devices.iter().find(|d| d.id == device_db_id) {
                    group_device_ids.push(device.device_id.clone());
                    synced_device_ids.insert(device.device_id.clone());
                }
            }

            // Only include groups with at least 2 devices
            if group_device_ids.len() >= 2 {
                synchronized.push(group_device_ids);
            }
        }

        // Find devices not in any sync group
        let not_synchronized: Vec<String> = all_device_ids
            .into_iter()
            .filter(|device_id| !synced_device_ids.contains(device_id))
            .collect();

        Ok(DeviceSyncStatus {
            synchronized,
            not_synchronized,
        })
    }

    /// Synchronize devices together and stop synchronization for specified devices
    pub async fn update_sync_groups(
        &self,
        user_id: i64,
        synchronize: Vec<Vec<String>>,
        stop_synchronize: Vec<String>,
    ) -> AppResult<DeviceSyncStatus> {
        // First, handle stop-synchronize requests
        for device_id in stop_synchronize {
            // Find the device by device_id string
            if let Some(device) = self
                .device_repo
                .find_by_device_id(user_id, &device_id)
                .await?
            {
                // Remove from any sync group
                self.device_sync_repo
                    .remove_device_from_group(device.id)
                    .await?;
            }
        }

        // Handle synchronize requests
        for device_group in synchronize {
            if device_group.len() < 2 {
                continue; // Need at least 2 devices to sync
            }

            // Convert device_id strings to database IDs
            let mut device_db_ids = Vec::new();
            for device_id in &device_group {
                if let Some(device) = self
                    .device_repo
                    .find_by_device_id(user_id, device_id)
                    .await?
                {
                    device_db_ids.push(device.id);
                } else {
                    return Err(AppError::BadRequest(format!(
                        "Device '{}' not found",
                        device_id
                    )));
                }
            }

            // Find existing sync groups for these devices
            let mut existing_groups: HashMap<i64, Vec<i64>> = HashMap::new();
            for &device_db_id in &device_db_ids {
                if let Some(group_id) = self.device_sync_repo.get_device_group(device_db_id).await?
                {
                    existing_groups
                        .entry(group_id)
                        .or_default()
                        .push(device_db_id);
                }
            }

            // Determine the target group
            let target_group_id = if existing_groups.is_empty() {
                // No existing groups - create a new one
                self.device_sync_repo.create_group(user_id).await?
            } else {
                // Use the first existing group as target
                *existing_groups.keys().next().unwrap()
            };

            // Merge other groups into the target group
            for (&group_id, _) in existing_groups.iter() {
                if group_id != target_group_id {
                    self.device_sync_repo
                        .merge_groups(target_group_id, group_id)
                        .await?;
                }
            }

            // Add all devices to the target group
            for &device_db_id in &device_db_ids {
                // Check if device is already in this group
                if let Some(current_group) =
                    self.device_sync_repo.get_device_group(device_db_id).await?
                {
                    if current_group == target_group_id {
                        continue; // Already in this group
                    }
                }

                // Add to target group (or update if in different group)
                // First remove from any existing group
                self.device_sync_repo
                    .remove_device_from_group(device_db_id)
                    .await?;
                // Then add to target group
                self.device_sync_repo
                    .add_device_to_group(target_group_id, device_db_id)
                    .await?;
            }
        }

        // Return updated sync status
        self.get_sync_status(user_id).await
    }
}
