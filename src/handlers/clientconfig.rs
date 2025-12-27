use serde::Serialize;
use warp::{reply::json, Rejection, Reply};

#[derive(Debug, Serialize)]
pub struct ClientConfig {
    pub mygpo: MyGpoConfig,
    pub update_timeout: i64,
}

#[derive(Debug, Serialize)]
pub struct MyGpoConfig {
    pub base_url: String,
}

pub async fn get_client_config(base_url: String) -> Result<impl Reply, Rejection> {
    let config = ClientConfig {
        mygpo: MyGpoConfig { base_url },
        update_timeout: 604800, // 7 days in seconds
    };

    Ok(json(&config))
}
