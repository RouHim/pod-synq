use std::sync::Arc;

use crate::config::Config;
use crate::error::AppResult;
use crate::models::PodcastMetadata;
use crate::repository::PodcastRepository;

pub struct PodcastService {
    podcast_repo: Arc<PodcastRepository>,
    config: Config,
}

impl PodcastService {
    pub fn new(podcast_repo: Arc<PodcastRepository>, config: Config) -> Self {
        Self {
            podcast_repo,
            config,
        }
    }

    /// Get podcast metadata for URLs, with fallback for missing podcasts
    pub async fn get_metadata_for_urls(&self, urls: &[String]) -> AppResult<Vec<PodcastMetadata>> {
        let podcasts = self.podcast_repo.get_by_urls(urls).await?;

        // Create a map of URL -> Podcast for fast lookup
        let podcast_map: std::collections::HashMap<_, _> =
            podcasts.into_iter().map(|p| (p.url.clone(), p)).collect();

        // Build metadata list, creating fallback entries for missing podcasts
        let mut result = Vec::new();
        for url in urls {
            let metadata = if let Some(podcast) = podcast_map.get(url) {
                podcast.to_metadata(&self.config.base_url)
            } else {
                // Fallback for podcasts not in database
                self.create_fallback_metadata(url)
            };
            result.push(metadata);
        }

        Ok(result)
    }

    /// Create minimal metadata when podcast not in database
    fn create_fallback_metadata(&self, url: &str) -> PodcastMetadata {
        PodcastMetadata {
            url: url.to_string(),
            title: url
                .split('/')
                .next_back()
                .unwrap_or("Unknown Podcast")
                .to_string(),
            description: String::new(),
            subscribers: 0,
            logo_url: None,
            website: None,
            mygpo_link: format!(
                "{}/podcast/{}",
                self.config.base_url,
                urlencoding::encode(url)
            ),
        }
    }
}
