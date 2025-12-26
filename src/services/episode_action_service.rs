use crate::{
    error::{AppError, AppResult},
    models::{EpisodeAction, EpisodeActionQuery},
    repository::EpisodeActionRepository,
};

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
    ) -> AppResult<Vec<EpisodeAction>> {
        Ok(self
            .action_repo
            .list(user_id, query)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?)
    }

    pub async fn upload_episode_actions(&self, actions: Vec<EpisodeAction>) -> AppResult<()> {
        let count = actions.len();
        self.action_repo
            .upload(actions)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!("Uploaded {} episode actions", count);
        Ok(())
    }

    pub async fn count_actions(&self, user_id: Option<i64>) -> AppResult<i64> {
        Ok(self
            .action_repo
            .count(user_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?)
    }
}
