pub mod admin;
mod health;
mod v1;

use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::middleware::tenant::tenant_middleware;
use crate::state::AppState;

pub fn app_router(state: AppState) -> Router {
    let v1 = Router::new()
        .merge(v1::router())
        .layer(from_fn_with_state(state.clone(), tenant_middleware));
    Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .nest("/v1", v1)
        .nest("/admin", admin::router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
