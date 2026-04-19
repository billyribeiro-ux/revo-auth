mod account;
mod email;
mod magic;
mod oauth;
mod org;
mod passkey;
mod password;
pub mod session;
mod signin;
mod signout;
mod signup;
mod totp;

use axum::middleware;
use axum::Router;

use crate::middleware::csrf::csrf_middleware;
use crate::middleware::rate_limit;
use crate::state::AppState;

/// The v1 router wires each sub-router with its spec-mandated rate-limit
/// layer. Sub-routers that contain multiple paths where only a subset needs
/// the stricter limit are split here into per-route branches.
pub fn router() -> Router<AppState> {
    // /v1/signup — 3 req/hour/IP
    let signup = signup::router().layer(rate_limit::signup_layer());
    // /v1/signin — 5 req/min/IP
    let signin = signin::router().layer(rate_limit::signin_layer());
    // /v1/password/reset/request — 3 req/hour/IP (reset_confirm keeps global)
    let password = password::router().layer(rate_limit::password_reset_request_layer());
    // /v1/magic/request + /v1/magic/verify — 5 req/hour/IP on request
    let magic = magic::router().layer(rate_limit::magic_request_layer());
    // /v1/oauth/*/callback — 10 req/min/IP (authorize keeps global)
    let oauth = oauth::router().layer(rate_limit::oauth_callback_layer());

    Router::new()
        .merge(signup)
        .merge(signin)
        .merge(signout::router())
        .merge(session::router())
        .merge(password)
        .merge(email::router())
        .merge(oauth)
        .merge(passkey::router())
        .merge(totp::router())
        .merge(magic)
        .merge(account::router())
        .merge(org::router())
        .layer(middleware::from_fn(csrf_middleware))
        .layer(rate_limit::v1_global_layer())
}
