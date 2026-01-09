//! Data Transfer Objects (DTOs) for API requests and responses
//!
//! This module contains all structs used for serializing/deserializing
//! data in HTTP request and response bodies.

use serde::{Deserialize, Serialize};

use crate::models::PodcastMetadata;

// ============================================================================
// Auth DTOs
// ============================================================================

/// Request body for logout endpoint
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    #[serde(default)]
    #[allow(dead_code)]
    pub session_id: Option<String>,
}

// ============================================================================
// Client Config DTOs
// ============================================================================

/// Client configuration response
#[derive(Debug, Serialize)]
pub struct ClientConfig {
    pub mygpo: MyGpoConfig,
    pub update_timeout: i64,
}

/// gpodder service URL configuration
#[derive(Debug, Serialize)]
pub struct MyGpoConfig {
    pub base_url: String,
}

// ============================================================================
// Device DTOs
// ============================================================================

/// Device information in list response
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub caption: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub subscriptions: i64,
}

/// Request body for updating a device
#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub caption: Option<String>,
    #[serde(rename = "type")]
    pub device_type: Option<String>,
}

/// Query parameters for device updates endpoint
#[derive(Debug, Deserialize)]
pub struct UpdatesQueryParams {
    pub since: Option<i64>,
    pub include_actions: Option<bool>,
}

/// Response for device updates endpoint
#[derive(Debug, Serialize)]
pub struct DeviceUpdatesResponse {
    pub add: Vec<PodcastMetadata>,
    pub remove: Vec<String>,
    pub updates: Vec<serde_json::Value>,
    pub timestamp: i64,
}

// ============================================================================
// Episode Action DTOs
// ============================================================================

/// Query parameters for episode actions endpoint
#[derive(Debug, Deserialize)]
pub struct EpisodeActionQueryParams {
    pub since: Option<i64>,
    pub podcast: Option<String>,
    pub device: Option<String>,
    pub aggregated: Option<bool>,
}

/// Single episode action in response
#[derive(Debug, Serialize)]
pub struct EpisodeActionResponse {
    pub podcast: String,
    pub episode: String,
    pub action: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    pub device: String,
}

/// Response for get episode actions endpoint
#[derive(Debug, Serialize)]
pub struct EpisodeActionsResult {
    pub actions: Vec<EpisodeActionResponse>,
    pub timestamp: i64,
}

/// Request body for uploading episode actions
#[derive(Debug, Deserialize)]
pub struct EpisodeActionUpload {
    pub podcast: String,
    pub episode: String,
    pub device: String,
    pub action: String,
    pub timestamp: i64,
    pub started: Option<i64>,
    pub position: Option<i64>,
    pub total: Option<i64>,
}

// ============================================================================
// Settings DTOs
// ============================================================================

/// Query parameters for settings endpoints
#[derive(Debug, Deserialize)]
pub struct SettingsQueryParams {
    pub podcast: Option<String>,
    pub device: Option<String>,
    pub episode: Option<String>,
}

// ============================================================================
// Subscription DTOs
// ============================================================================

/// Response for subscription list/upload endpoints
#[derive(Debug, Serialize)]
pub struct SubscriptionListResponse {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub update_urls: Vec<[String; 2]>,
}

/// Request body for uploading subscription changes
#[derive(Debug, Deserialize)]
pub struct SubscriptionUploadRequest {
    pub add: Option<Vec<String>>,
    pub remove: Option<Vec<String>>,
    pub timestamp: Option<i64>,
}

/// Query parameters for subscription endpoints
#[derive(Debug, Deserialize)]
pub struct SubscriptionQueryParams {
    pub since: Option<i64>,
}
