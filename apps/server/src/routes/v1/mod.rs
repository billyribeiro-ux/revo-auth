mod account;
mod email;
mod magic;
mod oauth;
mod org;
mod passkey;
mod password;
mod session;
mod signin;
mod signout;
mod signup;
mod totp;

use axum::middleware;
use axum::Router;

use crate::middleware::csrf::csrf_middleware;
use crate::middleware::rate_limit::v1_rate_limit_layer;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(signup::router())
        .merge(signin::router())
        .merge(signout::router())
        .merge(session::router())
        .merge(password::router())
        .merge(email::router())
        .merge(oauth::router())
        .merge(passkey::router())
        .merge(totp::router())
        .merge(magic::router())
        .merge(account::router())
        .merge(org::router())
        .layer(middleware::from_fn(csrf_middleware))
        .layer(v1_rate_limit_layer())
}
