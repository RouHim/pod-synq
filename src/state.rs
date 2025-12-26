use sqlx::SqlitePool;
use std::sync::Arc;

use crate::services::{
    AdminSessionService, DeviceService, EpisodeActionService, SubscriptionService, UserService,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub user_service: Arc<UserService>,
    pub device_service: Arc<DeviceService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub episode_action_service: Arc<EpisodeActionService>,
    pub admin_session_service: Arc<AdminSessionService>,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        let user_repo = crate::repository::UserRepository::new(pool.clone());
        let device_repo = crate::repository::DeviceRepository::new(pool.clone());
        let sub_repo = crate::repository::SubscriptionRepository::new(pool.clone());
        let action_repo = crate::repository::EpisodeActionRepository::new(pool.clone());
        let session_repo = crate::repository::AdminSessionRepository::new(pool.clone());

        let user_service = Arc::new(UserService::new(user_repo));
        let device_service = Arc::new(DeviceService::new(device_repo));
        let subscription_service = Arc::new(SubscriptionService::new(sub_repo));
        let episode_action_service = Arc::new(EpisodeActionService::new(action_repo));
        let admin_session_service = Arc::new(AdminSessionService::new(session_repo));

        Self {
            pool,
            user_service,
            device_service,
            subscription_service,
            episode_action_service,
            admin_session_service,
        }
    }
}
