use bytes::Bytes;
use warp::{
    reject,
    reply::{self, json},
    Rejection, Reply,
};

use crate::error::AppError;
use crate::middleware::AuthorizedContext;
use crate::models::{
    SubscriptionChanges, SubscriptionListResponse, SubscriptionQueryParams,
    SubscriptionUploadRequest,
};
use crate::state::AppState;
use crate::utils::{to_opml, to_txt};

pub async fn get_subscriptions(
    device_id: String,
    params: SubscriptionQueryParams,
    auth: AuthorizedContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    let db_device_id = state
        .device_service
        .get_or_create_device(auth.user_id, &device_id, None, None)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    if let Some(since) = params.since {
        let (add, remove) = state
            .subscription_service
            .get_changes_since(auth.user_id, db_device_id, since)
            .await
            .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

        Ok(json(&SubscriptionListResponse {
            add,
            remove,
            timestamp: chrono::Utc::now().timestamp(),
            update_urls: vec![],
        }))
    } else {
        let subscriptions = state
            .subscription_service
            .get_subscriptions(auth.user_id, db_device_id)
            .await
            .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

        Ok(json(&subscriptions))
    }
}

pub async fn upload_subscriptions(
    device_id: String,
    auth: AuthorizedContext,
    state: AppState,
    req: SubscriptionUploadRequest,
) -> Result<impl Reply, Rejection> {
    let db_device_id = state
        .device_service
        .get_or_create_device(auth.user_id, &device_id, None, None)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    // Sanitize URLs
    let (sanitized_add, mut add_updates) =
        crate::utils::sanitize_urls(req.add.as_deref().unwrap_or_default());
    let (sanitized_remove, mut remove_updates) =
        crate::utils::sanitize_urls(req.remove.as_deref().unwrap_or_default());

    // Check for conflicts (same URL in both add and remove)
    for add_url in &sanitized_add {
        if !add_url.is_empty() && sanitized_remove.contains(add_url) {
            return Err(reject::custom(AppError::Conflict(format!(
                "URL cannot be both added and removed: {}",
                add_url
            ))));
        }
    }

    // Combine update_urls from both add and remove
    let mut all_updates = Vec::new();
    all_updates.append(&mut add_updates);
    all_updates.append(&mut remove_updates);

    let changes = SubscriptionChanges {
        add: sanitized_add,
        remove: sanitized_remove,
        timestamp: req
            .timestamp
            .unwrap_or_else(|| chrono::Utc::now().timestamp()),
    };

    state
        .subscription_service
        .upload_changes(auth.user_id, db_device_id, changes)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    Ok(json(&SubscriptionListResponse {
        add: vec![],
        remove: vec![],
        timestamp: chrono::Utc::now().timestamp(),
        update_urls: all_updates,
    }))
}

pub async fn get_subscriptions_simple(
    device_id: String,
    format: String,
    auth: AuthorizedContext,
    state: AppState,
) -> Result<Box<dyn Reply + Send>, Rejection> {
    let db_device_id = state
        .device_service
        .get_or_create_device(auth.user_id, &device_id, None, None)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let subscriptions = state
        .subscription_service
        .get_subscriptions(auth.user_id, db_device_id)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let reply: Box<dyn Reply + Send> = match format.as_str() {
        "json" => Box::new(json(&subscriptions)),
        "opml" => Box::new(reply::with_header(
            to_opml(&subscriptions),
            "content-type",
            "text/xml",
        )),
        "txt" => Box::new(reply::with_header(
            to_txt(&subscriptions),
            "content-type",
            "text/plain",
        )),
        _ => {
            return Err(reject::custom(AppError::InvalidFormat(format)));
        }
    };

    Ok(reply)
}

pub async fn get_all_subscriptions_simple(
    format: String,
    auth: AuthorizedContext,
    state: AppState,
) -> Result<Box<dyn Reply + Send>, Rejection> {
    let subscriptions = state
        .subscription_service
        .get_all_subscriptions(auth.user_id)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let reply: Box<dyn Reply + Send> = match format.as_str() {
        "json" => Box::new(json(&subscriptions)),
        "opml" => Box::new(reply::with_header(
            to_opml(&subscriptions),
            "content-type",
            "text/xml",
        )),
        "txt" => Box::new(reply::with_header(
            to_txt(&subscriptions),
            "content-type",
            "text/plain",
        )),
        _ => {
            return Err(reject::custom(AppError::InvalidFormat(format)));
        }
    };

    Ok(reply)
}

pub async fn upload_subscriptions_simple(
    device_id: String,
    format: String,
    auth: AuthorizedContext,
    state: AppState,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    let db_device_id = state
        .device_service
        .get_or_create_device(auth.user_id, &device_id, None, None)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let body_str = String::from_utf8(body.to_vec())
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let podcast_urls = match format.as_str() {
        "json" => {
            let urls: Vec<String> = serde_json::from_str(&body_str)
                .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;
            urls
        }
        "txt" => body_str
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect(),
        "opml" => {
            let urls: Vec<String> = body_str
                .lines()
                .filter_map(|line| {
                    if let Some(start) = line.find("xmlUrl=\"") {
                        if let Some(end) = line[start + 8..].find('"') {
                            return Some(line[start + 8..start + 8 + end].to_string());
                        }
                    }
                    None
                })
                .collect();
            urls
        }
        _ => {
            return Err(reject::custom(AppError::InvalidFormat(format)));
        }
    };

    state
        .subscription_service
        .set_subscriptions(auth.user_id, db_device_id, podcast_urls)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    Ok(reply::with_header("", "content-type", "text/plain"))
}
