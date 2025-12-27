use sqlx::SqlitePool;
use std::sync::Arc;

use crate::services::{
    DeviceService, DeviceSyncService, EpisodeActionService, FavoriteService, SessionService,
    SettingService, SubscriptionService, UserService,
};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub device_service: Arc<DeviceService>,
    pub device_sync_service: Arc<DeviceSyncService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub episode_action_service: Arc<EpisodeActionService>,
    pub setting_service: Arc<SettingService>,
    pub session_service: Arc<SessionService>,
    pub favorite_service: Arc<FavoriteService>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let user_repo = crate::repository::UserRepository::new(pool.clone());
        let device_repo = crate::repository::DeviceRepository::new(pool.clone());
        let device_sync_repo = crate::repository::DeviceSyncRepository::new(pool.clone());
        let sub_repo = crate::repository::SubscriptionRepository::new(pool.clone());
        let action_repo = crate::repository::EpisodeActionRepository::new(pool.clone());
        let setting_repo = crate::repository::SettingRepository::new(pool.clone());
        let session_repo = crate::repository::SessionRepository::new(pool.clone());
        let favorite_repo = crate::repository::FavoriteRepository::new(pool.clone());

        let user_service = Arc::new(UserService::new(user_repo));
        let device_service = Arc::new(DeviceService::new(device_repo.clone()));
        let device_sync_service = Arc::new(DeviceSyncService::new(device_sync_repo, device_repo));
        let subscription_service = Arc::new(SubscriptionService::new(sub_repo));
        let episode_action_service = Arc::new(EpisodeActionService::new(action_repo));
        let setting_service = Arc::new(SettingService::new(setting_repo));
        let session_service = Arc::new(SessionService::new(session_repo));
        let favorite_service = Arc::new(FavoriteService::new(favorite_repo));

        Self {
            user_service,
            device_service,
            device_sync_service,
            subscription_service,
            episode_action_service,
            setting_service,
            session_service,
            favorite_service,
        }
    }
}
