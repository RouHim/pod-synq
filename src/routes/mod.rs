use warp::Filter;

use crate::handlers::{auth, devices, episodes, subscriptions};
use crate::middleware::{with_auth, AuthService};
use crate::state::AppState;

pub fn create_routes(
    auth_service: AuthService,
    state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let state_filter = warp::any().map(move || state.clone());

    let auth_filter = with_auth(auth_service.clone());

    let login = warp::get()
        .and(warp::path!("api" / "2" / "auth" / String / "login.json"))
        .and(auth_filter.clone())
        .and_then(auth::login);

    let logout = warp::post()
        .and(warp::path!("api" / "2" / "auth" / String / "logout.json"))
        .and(auth_filter.clone())
        .and(warp::body::json())
        .and_then(auth::logout);

    let list_devices = warp::get()
        .and(warp::path!("api" / "2" / "devices" / String / ".json"))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(devices::list_devices);

    let update_device = warp::post()
        .and(warp::path!(
            "api" / "2" / "devices" / String / String / ".json"
        ))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and(warp::body::json())
        .and_then(devices::update_device);

    let get_subscriptions = warp::get()
        .and(warp::path!(
            "api" / "2" / "subscriptions" / String / String / ".json"
        ))
        .and(auth_filter.clone())
        .and(state_filter.clone())
        .and_then(subscriptions::get_subscriptions);

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

    login
        .or(logout)
        .or(list_devices)
        .or(update_device)
        .or(get_subscriptions)
        .or(upload_subscriptions)
        .or(get_episode_actions)
        .or(upload_episode_actions)
        .recover(crate::error::handle_rejection)
}
