use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct DeviceSyncStatus {
    pub synchronized: Vec<Vec<String>>,
    #[serde(rename = "not-synchronized")]
    pub not_synchronized: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceSyncRequest {
    #[serde(default)]
    pub synchronize: Vec<Vec<String>>,
    #[serde(rename = "stop-synchronize", default)]
    pub stop_synchronize: Vec<String>,
}
