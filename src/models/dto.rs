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
pub struct LogoutRequest {}

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

/// Deserializes a gPodder timestamp from either an ISO 8601 string or a Unix
/// epoch integer into seconds since the epoch.
fn deserialize_flexible_timestamp<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use chrono::{NaiveDateTime, Utc};
    use serde::de::Error;
    use serde_json::Value;

    match Value::deserialize(deserializer)? {
        Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| D::Error::custom("invalid timestamp")),
        Value::String(s) => NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
            .map(|dt| dt.and_utc().timestamp())
            .or_else(|_| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc).timestamp())
            })
            .map_err(|e| D::Error::custom(format!("invalid timestamp: {e}"))),
        _ => Err(D::Error::custom("timestamp must be a string or integer")),
    }
}

/// Request body for uploading episode actions
#[derive(Debug, Deserialize)]
pub struct EpisodeActionUpload {
    pub podcast: String,
    pub episode: String,
    #[serde(default)]
    pub device: String,
    pub action: String,
    #[serde(deserialize_with = "deserialize_flexible_timestamp")]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Given an integer Unix epoch timestamp, deserialization keeps the
    /// integer value unchanged.
    #[test]
    fn deserializes_integer_timestamp_as_epoch_seconds() {
        let json = r#"{"podcast":"p","episode":"e","action":"new","timestamp":1700000000}"#;

        let upload: EpisodeActionUpload =
            serde_json::from_str(json).expect("integer timestamp must deserialize");

        assert_eq!(upload.timestamp, 1_700_000_000);
    }

    /// Given an ISO 8601 string without offset, deserialization parses it as
    /// UTC and yields the matching epoch seconds.
    #[test]
    fn deserializes_iso_string_without_offset_as_utc_epoch_seconds() {
        let json =
            r#"{"podcast":"p","episode":"e","action":"new","timestamp":"2023-11-14T22:13:20"}"#;

        let upload: EpisodeActionUpload =
            serde_json::from_str(json).expect("naive ISO timestamp must deserialize");

        assert_eq!(upload.timestamp, 1_700_000_000);
    }

    /// Given an RFC 3339 string with a UTC offset, deserialization converts
    /// it to the correct epoch seconds.
    #[test]
    fn deserializes_rfc3339_string_with_offset_to_epoch_seconds() {
        let json = r#"{"podcast":"p","episode":"e","action":"new","timestamp":"2023-11-14T23:13:20+01:00"}"#;

        let upload: EpisodeActionUpload =
            serde_json::from_str(json).expect("RFC 3339 timestamp must deserialize");

        assert_eq!(upload.timestamp, 1_700_000_000);
    }

    /// Given a null timestamp, deserialization fails instead of defaulting.
    #[test]
    fn rejects_null_timestamp_with_deserialization_error() {
        let json = r#"{"podcast":"p","episode":"e","action":"new","timestamp":null}"#;

        let result: Result<EpisodeActionUpload, _> = serde_json::from_str(json);

        assert!(result.is_err(), "null timestamp must not deserialize");
    }

    /// Given a payload omitting optional fields, `device` defaults to an
    /// empty string and started/position/total remain None.
    #[test]
    fn defaults_optional_fields_when_omitted() {
        let json = r#"{"podcast":"p","episode":"e","action":"new","timestamp":1700000000}"#;

        let upload: EpisodeActionUpload =
            serde_json::from_str(json).expect("payload without optionals must deserialize");

        assert_eq!(upload.device, "");
        assert!(upload.started.is_none());
        assert!(upload.position.is_none());
        assert!(upload.total.is_none());
    }
}
