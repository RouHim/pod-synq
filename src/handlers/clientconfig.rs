use warp::{reply::json, Rejection, Reply};

use crate::constants::CLIENT_UPDATE_TIMEOUT_SECS;
use crate::models::{ClientConfig, MyGpoConfig};

pub async fn get_client_config(base_url: String) -> Result<impl Reply, Rejection> {
    let config = ClientConfig {
        mygpo: MyGpoConfig { base_url },
        update_timeout: CLIENT_UPDATE_TIMEOUT_SECS,
    };

    Ok(json(&config))
}
