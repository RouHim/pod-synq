use crate::error::AppResult;
use crate::models::{EpisodeAction, EpisodeActionQuery};
use crate::repository::traits::EpisodeActionRepositoryTrait;
use crate::repository::{EpisodeActionRepository, EpisodeActionWithDevice};

#[derive(Clone)]
pub struct EpisodeActionService {
    action_repo: EpisodeActionRepository,
}

impl EpisodeActionService {
    pub fn new(action_repo: EpisodeActionRepository) -> Self {
        Self { action_repo }
    }

    pub async fn get_episode_actions(
        &self,
        user_id: i64,
        query: EpisodeActionQuery,
    ) -> AppResult<Vec<EpisodeActionWithDevice>> {
        self.action_repo.list(user_id, query).await
    }

    pub async fn get_actions_since(
        &self,
        user_id: i64,
        device_id: Option<i64>,
        podcast_url: Option<String>,
        since: i64,
    ) -> AppResult<Vec<EpisodeActionWithDevice>> {
        let query = EpisodeActionQuery {
            since: Some(since),
            podcast: podcast_url,
            device: device_id.map(|id| id.to_string()),
            aggregated: None,
        };
        self.action_repo.list(user_id, query).await
    }

    pub async fn upload_episode_actions(&self, actions: Vec<EpisodeAction>) -> AppResult<()> {
        let count = actions.len();
        self.action_repo.upload(actions).await?;
        tracing::info!("Uploaded {} episode actions", count);
        Ok(())
    }
}
