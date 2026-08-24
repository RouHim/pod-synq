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

/// Parses the request tail after the fixed route prefix into a single identifier.
/// Accepts `id`, `id.json`, and `id/.json`; rejects deeper paths.
async fn parse_id_tail(tail: warp::path::Tail) -> Result<String, warp::Rejection> {
    let id = tail.as_str();
    let id = id.strip_suffix(".json").unwrap_or(id);
    let id = id.strip_suffix('/').unwrap_or(id);
    if id.is_empty() || id.contains('/') {
        return Err(warp::reject::not_found());
    }
    Ok(id.to_string())
}

/// Validates the tail of a single-identifier endpoint: nothing or `.json`.
async fn validate_unit_tail(tail: warp::path::Tail) -> Result<(), warp::Rejection> {
    match tail.as_str() {
        "" | ".json" => Ok(()),
        _ => Err(warp::reject::not_found()),
    }
}

/// Strips a trailing `.json` extension from a captured path segment.
fn strip_json_ext(segment: &str) -> &str {
    segment.strip_suffix(".json").unwrap_or(segment)
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
        .and(warp::path!("api" / "2" / "devices" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, tail, auth: AuthContext, state| async move {
                validate_unit_tail(tail).await?;
                let auth = authorize(auth, strip_json_ext(&username))?;
                devices::list_devices(auth, state).await
            },
        );

    let update_device = warp::post()
        .and(warp::path!("api" / "2" / "devices" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, tail, auth: AuthContext, state, req| async move {
                let device_id = parse_id_tail(tail).await?;
                let auth = authorize(auth, &username)?;
                devices::update_device(device_id, auth, state, req).await
            },
        );

    let get_device_updates = warp::get()
        .and(warp::path!("api" / "2" / "updates" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(warp::query::<UpdatesQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, tail, params, auth: AuthContext, state| async move {
                let device_id = parse_id_tail(tail).await?;
                let auth = authorize(auth, &username)?;
                devices::get_device_updates(device_id, params, auth, state).await
            },
        );

    let get_subscriptions = warp::get()
        .and(warp::path!("api" / "2" / "subscriptions" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(warp::query::<SubscriptionQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, tail, params, auth: AuthContext, state| async move {
                let device_id = parse_id_tail(tail).await?;
                let auth = authorize(auth, &username)?;
                subscriptions::get_subscriptions(device_id, params, auth, state).await
            },
        );

    let upload_subscriptions = warp::post()
        .and(warp::path!("api" / "2" / "subscriptions" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, tail, auth: AuthContext, state, req| async move {
                let device_id = parse_id_tail(tail).await?;
                let auth = authorize(auth, &username)?;
                subscriptions::upload_subscriptions(device_id, auth, state, req).await
            },
        );

    let get_episode_actions = warp::get()
        .and(warp::path!("api" / "2" / "episodes" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(warp::query::<EpisodeActionQueryParams>())
        .and(state_filter.clone())
        .and_then(
            |username: String, tail, auth: AuthContext, params, state| async move {
                validate_unit_tail(tail).await?;
                let auth = authorize(auth, strip_json_ext(&username))?;
                episodes::get_episode_actions(auth, params, state).await
            },
        );

    let upload_episode_actions = warp::post()
        .and(warp::path!("api" / "2" / "episodes" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, tail, auth: AuthContext, state, actions| async move {
                validate_unit_tail(tail).await?;
                let auth = authorize(auth, strip_json_ext(&username))?;
                episodes::upload_episode_actions(auth, state, actions).await
            },
        );

    let get_settings = warp::get()
        .and(warp::path!("api" / "2" / "settings" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(warp::query::<SettingsQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, tail, params, auth: AuthContext, state| async move {
                let scope = parse_id_tail(tail).await?;
                let auth = authorize(auth, &username)?;
                settings::get_settings(scope, params, auth, state).await
            },
        );

    let save_settings = warp::post()
        .and(warp::path!("api" / "2" / "settings" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(warp::query::<SettingsQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, tail, params, auth: AuthContext, state, req| async move {
                let scope = parse_id_tail(tail).await?;
                let auth = authorize(auth, &username)?;
                settings::save_settings(scope, params, auth, state, req).await
            },
        );

    let config_clone = config.clone();
    let get_favorites = warp::get()
        .and(warp::path!("api" / "2" / "favorites" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::any().map(move || config_clone.clone()))
        .and_then(
            |username: String, tail, auth: AuthContext, state, config| async move {
                validate_unit_tail(tail).await?;
                let auth = authorize(auth, strip_json_ext(&username))?;
                favorites::get_favorites(auth, state, config).await
            },
        );

    let get_sync_devices = warp::get()
        .and(warp::path!("api" / "2" / "sync-devices" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(
            |username: String, tail, auth: AuthContext, state| async move {
                validate_unit_tail(tail).await?;
                let auth = authorize(auth, strip_json_ext(&username))?;
                device_sync::get_sync_status(auth, state).await
            },
        );

    let update_sync_devices = warp::post()
        .and(warp::path!("api" / "2" / "sync-devices" / ..))
        .and(warp::path::param::<String>())
        .and(warp::path::tail())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, tail, auth: AuthContext, state, request| async move {
                validate_unit_tail(tail).await?;
                let auth = authorize(auth, strip_json_ext(&username))?;
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
