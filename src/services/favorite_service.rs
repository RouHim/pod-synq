use crate::error::AppResult;
use crate::models::{FavoriteMetadata, FavoriteResponse};
use crate::repository::FavoriteRepository;

#[derive(Clone)]
pub struct FavoriteService {
    favorite_repo: FavoriteRepository,
}

impl FavoriteService {
    pub fn new(favorite_repo: FavoriteRepository) -> Self {
        Self { favorite_repo }
    }

    /// Add an episode to favorites
    pub async fn add_favorite(
        &self,
        user_id: i64,
        metadata: &FavoriteMetadata<'_>,
    ) -> AppResult<i64> {
        let id = self.favorite_repo.add_favorite(user_id, metadata).await?;
        Ok(id)
    }

    /// Remove an episode from favorites
    pub async fn remove_favorite(&self, user_id: i64, episode_url: &str) -> AppResult<()> {
        self.favorite_repo
            .remove_favorite(user_id, episode_url)
            .await?;
        Ok(())
    }

    /// Get all favorites for a user
    pub async fn get_user_favorites(
        &self,
        user_id: i64,
        base_url: &str,
    ) -> AppResult<Vec<FavoriteResponse>> {
        let favorites = self.favorite_repo.get_user_favorites(user_id).await?;
        let responses: Vec<FavoriteResponse> =
            favorites.iter().map(|f| f.to_response(base_url)).collect();
        Ok(responses)
    }

    /// Check if an episode is favorited
    #[allow(dead_code)]
    pub async fn is_favorite(&self, user_id: i64, episode_url: &str) -> AppResult<bool> {
        let is_fav = self.favorite_repo.is_favorite(user_id, episode_url).await?;
        Ok(is_fav)
    }
}
