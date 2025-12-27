use sqlx::SqlitePool;
use std::sync::Arc;

use crate::services::{
    DeviceService, EpisodeActionService, SettingService, SubscriptionService, UserService,
};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub device_service: Arc<DeviceService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub episode_action_service: Arc<EpisodeActionService>,
    pub setting_service: Arc<SettingService>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let user_repo = crate::repository::UserRepository::new(pool.clone());
        let device_repo = crate::repository::DeviceRepository::new(pool.clone());
        let sub_repo = crate::repository::SubscriptionRepository::new(pool.clone());
        let action_repo = crate::repository::EpisodeActionRepository::new(pool.clone());
        let setting_repo = crate::repository::SettingRepository::new(pool.clone());

        let user_service = Arc::new(UserService::new(user_repo));
        let device_service = Arc::new(DeviceService::new(device_repo));
        let subscription_service = Arc::new(SubscriptionService::new(sub_repo));
        let episode_action_service = Arc::new(EpisodeActionService::new(action_repo));
        let setting_service = Arc::new(SettingService::new(setting_repo));

        Self {
            user_service,
            device_service,
            subscription_service,
            episode_action_service,
            setting_service,
        }
    }
}
