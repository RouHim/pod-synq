use warp::Filter;

use crate::config::Config;
use crate::error::AppError;
use crate::handlers::{
    auth, clientconfig, device_sync, devices, episodes, favorites, settings, subscriptions,
};
use crate::middleware::{with_auth, AuthContext, AuthService, AuthorizedContext};
use crate::models::{
    EpisodeActionQueryParams, LogoutRequest, SettingsQueryParams, SubscriptionQueryParams,
    UpdatesQueryParams,
};
use crate::state::AppState;

/// Check if the authenticated user matches the username in the path
fn authorize(auth: AuthContext, username: &str) -> Result<AuthorizedContext, warp::Rejection> {
    if auth.username != username {
        return Err(warp::reject::custom(AppError::Authorization));
    }
    Ok(AuthorizedContext {
        user_id: auth.user_id,
        username: auth.username,
    })
}

pub fn create_routes(
    auth_service: AuthService,
    state: AppState,
    config: Config,
) -> impl Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    let auth_filter = with_auth(auth_service.clone());

    let base_url = config.base_url.clone();
    let client_config = warp::get()
        .and(warp::path!("clientconfig.json"))
        .and(warp::any().map(move || base_url.clone()))
        .and_then(clientconfig::get_client_config);

    let login = warp::post()
        .and(warp::path!("api" / "2" / "auth" / String / "login.json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(
            warp::body::json()
                .or(warp::any().map(|| serde_json::Value::Null))
                .unify(),
        )
        .and_then(
            |username, auth, state, _body: serde_json::Value| async move {
                auth::login(username, auth, state).await
            },
        );

    let logout = warp::post()
        .and(warp::path!("api" / "2" / "auth" / String / "logout.json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::header::optional::<String>("cookie"))
        .and(warp::body::json::<LogoutRequest>())
        .and_then(auth::logout);

    let list_devices = warp::get()
        .and(warp::path!("api" / "2" / "devices" / String))
        .and(warp::path::end())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username_with_ext: String, auth: AuthContext, state| async move {
                let username = username_with_ext.trim_end_matches(".json");
                let auth = authorize(auth, username)?;
                devices::list_devices(auth, state).await
            },
        );

    let update_device = warp::post()
        .and(warp::path!("api" / "2" / "devices" / String / String))
        .and(warp::path::end())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, device_id_with_ext: String, auth: AuthContext, state, req| async move {
                let username = username.trim_end_matches(".json");
                let device_id = device_id_with_ext.trim_end_matches(".json");
                let auth = authorize(auth, username)?;
                devices::update_device(device_id.to_string(), auth, state, req).await
            },
        );

    let get_device_updates = warp::get()
        .and(warp::path!(
            "api" / "2" / "updates" / String / String / ".json"
        ))
        .and(warp::query::<UpdatesQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, device_id: String, params, auth: AuthContext, state| async move {
                let auth = authorize(auth, &username)?;
                devices::get_device_updates(device_id, params, auth, state).await
            },
        );

    let get_subscriptions = warp::get()
        .and(warp::path!(
            "api" / "2" / "subscriptions" / String / String / ".json"
        ))
        .and(warp::query::<SubscriptionQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, device_id: String, params, auth: AuthContext, state| async move {
                let auth = authorize(auth, &username)?;
                subscriptions::get_subscriptions(device_id, params, auth, state).await
            },
        );

    let upload_subscriptions = warp::post()
        .and(warp::path!(
            "api" / "2" / "subscriptions" / String / String / ".json"
        ))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, device_id: String, auth: AuthContext, state, req| async move {
                let auth = authorize(auth, &username)?;
                subscriptions::upload_subscriptions(device_id, auth, state, req).await
            },
        );

    let get_episode_actions = warp::get()
        .and(warp::path!("api" / "2" / "episodes" / String / ".json"))
        .and(auth_filter.clone())
        .and(warp::query::<EpisodeActionQueryParams>())
        .and(state_filter.clone())
        .and_then(
            |username: String, auth: AuthContext, params, state| async move {
                let auth = authorize(auth, &username)?;
                episodes::get_episode_actions(auth, params, state).await
            },
        );

    let upload_episode_actions = warp::post()
        .and(warp::path!("api" / "2" / "episodes" / String / ".json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, auth: AuthContext, state, actions| async move {
                let auth = authorize(auth, &username)?;
                episodes::upload_episode_actions(auth, state, actions).await
            },
        );

    let get_settings = warp::get()
        .and(warp::path!(
            "api" / "2" / "settings" / String / String / ".json"
        ))
        .and(warp::query::<SettingsQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, scope: String, params, auth: AuthContext, state| async move {
                let auth = authorize(auth, &username)?;
                settings::get_settings(scope, params, auth, state).await
            },
        );

    let save_settings = warp::post()
        .and(warp::path!(
            "api" / "2" / "settings" / String / String / ".json"
        ))
        .and(warp::query::<SettingsQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, scope: String, params, auth: AuthContext, state, req| async move {
                let auth = authorize(auth, &username)?;
                settings::save_settings(scope, params, auth, state, req).await
            },
        );

    let config_clone = config.clone();
    let get_favorites = warp::get()
        .and(warp::path!("api" / "2" / "favorites" / String / ".json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::any().map(move || config_clone.clone()))
        .and_then(
            |username: String, auth: AuthContext, state, config| async move {
                let auth = authorize(auth, &username)?;
                favorites::get_favorites(auth, state, config).await
            },
        );

    let get_sync_devices = warp::get()
        .and(warp::path!("api" / "2" / "sync-devices" / String / ".json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(|username: String, auth: AuthContext, state| async move {
            let auth = authorize(auth, &username)?;
            device_sync::get_sync_status(auth, state).await
        });

    let update_sync_devices = warp::post()
        .and(warp::path!("api" / "2" / "sync-devices" / String / ".json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, auth: AuthContext, state, request| async move {
                let auth = authorize(auth, &username)?;
                device_sync::update_sync_groups(auth, state, request).await
            },
        );

    // Simple API routes (v1 style)
    let get_subscriptions_simple = warp::get()
        .and(warp::path!("subscriptions" / String / String / String))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, device_id: String, format: String, auth: AuthContext, state| async move {
                let auth = authorize(auth, &username)?;
                subscriptions::get_subscriptions_simple(device_id, format, auth, state).await
            },
        );

    let get_all_subscriptions_simple = warp::get()
        .and(warp::path!("subscriptions" / String / String))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, format: String, auth: AuthContext, state| async move {
                let auth = authorize(auth, &username)?;
                subscriptions::get_all_subscriptions_simple(format, auth, state).await
            },
        );

    let upload_subscriptions_simple = warp::put()
        .and(warp::path!("subscriptions" / String / String / String))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::bytes())
        .and_then(
            |username: String,
             device_id: String,
             format: String,
             auth: AuthContext,
             state,
             body| async move {
                let auth = authorize(auth, &username)?;
                subscriptions::upload_subscriptions_simple(device_id, format, auth, state, body)
                    .await
            },
        );

    client_config
        .or(login)
        .or(logout)
        .or(list_devices)
        .or(update_device)
        .or(get_device_updates)
        .or(get_sync_devices)
        .or(update_sync_devices)
        .or(get_subscriptions)
        .or(upload_subscriptions)
        .or(get_episode_actions)
        .or(upload_episode_actions)
        .or(get_settings)
        .or(save_settings)
        .or(get_favorites)
        .or(get_subscriptions_simple)
        .or(get_all_subscriptions_simple)
        .or(upload_subscriptions_simple)
        .recover(crate::error::handle_rejection)
}
