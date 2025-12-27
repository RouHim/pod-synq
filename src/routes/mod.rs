use warp::Filter;

use crate::handlers::{auth, devices, episodes, settings, subscriptions};
use crate::middleware::{with_auth, AuthService};
use crate::state::AppState;

pub fn create_routes(
    auth_service: AuthService,
    state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = std::convert::Infallible> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    let auth_filter = with_auth(auth_service.clone());

    let login = warp::post()
        .and(warp::path!("api" / "2" / "auth" / String / "login.json"))
        .and(auth_filter.clone())
        .and(
            warp::body::json()
                .or(warp::any().map(|| serde_json::Value::Null))
                .unify(),
        )
        .and_then(|username, auth, _body: serde_json::Value| async move {
            auth::login(username, auth).await
        });

    let logout = warp::post()
        .and(warp::path!("api" / "2" / "auth" / String / "logout.json"))
        .and(auth_filter.clone())
        .and(warp::body::json())
        .and_then(auth::logout);

    let list_devices = warp::get()
        .and(warp::path!("api" / "2" / "devices" / String))
        .and(warp::path::end())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(|username_with_ext: String, auth, state| async move {
            let username = username_with_ext.trim_end_matches(".json");
            devices::list_devices(username.to_string(), auth, state).await
        });

    let update_device = warp::post()
        .and(warp::path!("api" / "2" / "devices" / String / String))
        .and(warp::path::end())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(
            |username: String, device_id_with_ext: String, auth, state, req| async move {
                let device_id = device_id_with_ext.trim_end_matches(".json");
                devices::update_device(username, device_id.to_string(), auth, state, req).await
            },
        );

    let get_device_updates = warp::get()
        .and(warp::path!(
            "api" / "2" / "updates" / String / String / ".json"
        ))
        .and(warp::query::<devices::UpdatesQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(|username, device_id, params, auth, state| async move {
            devices::get_device_updates(username, device_id, params, auth, state).await
        });

    let get_subscriptions = warp::get()
        .and(warp::path!(
            "api" / "2" / "subscriptions" / String / String / ".json"
        ))
        .and(warp::query::<subscriptions::SubscriptionQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(|username, device_id, params, auth, state| async move {
            subscriptions::get_subscriptions(username, device_id, params, auth, state).await
        });

    let upload_subscriptions = warp::post()
        .and(warp::path!(
            "api" / "2" / "subscriptions" / String / String / ".json"
        ))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(subscriptions::upload_subscriptions);

    let get_episode_actions = warp::get()
        .and(warp::path!("api" / "2" / "episodes" / String / ".json"))
        .and(auth_filter.clone())
        .and(warp::query::<episodes::EpisodeActionQueryParams>())
        .and(state_filter.clone())
        .and_then(episodes::get_episode_actions);

    let upload_episode_actions = warp::post()
        .and(warp::path!("api" / "2" / "episodes" / String / ".json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(episodes::upload_episode_actions);

    let get_settings = warp::get()
        .and(warp::path!(
            "api" / "2" / "settings" / String / String / ".json"
        ))
        .and(warp::query::<settings::SettingsQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(|username, scope, params, auth, state| async move {
            settings::get_settings(username, scope, params, auth, state).await
        });

    let save_settings = warp::post()
        .and(warp::path!(
            "api" / "2" / "settings" / String / String / ".json"
        ))
        .and(warp::query::<settings::SettingsQueryParams>())
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(|username, scope, params, auth, state, req| async move {
            settings::save_settings(username, scope, params, auth, state, req).await
        });

    let get_subscriptions_simple = warp::get()
        .and(warp::path!("subscriptions" / String / String / String))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(subscriptions::get_subscriptions_simple);

    let get_all_subscriptions_simple = warp::get()
        .and(warp::path!("subscriptions" / String / String))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(subscriptions::get_all_subscriptions_simple);

    let upload_subscriptions_simple = warp::put()
        .and(warp::path!("subscriptions" / String / String / String))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::bytes())
        .and_then(subscriptions::upload_subscriptions_simple);

    login
        .or(logout)
        .or(list_devices)
        .or(update_device)
        .or(get_device_updates)
        .or(get_subscriptions)
        .or(upload_subscriptions)
        .or(get_episode_actions)
        .or(upload_episode_actions)
        .or(get_settings)
        .or(save_settings)
        .or(get_subscriptions_simple)
        .or(get_all_subscriptions_simple)
        .or(upload_subscriptions_simple)
        .recover(crate::error::handle_rejection)
}
