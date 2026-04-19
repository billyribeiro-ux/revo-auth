pub mod templates;
pub mod transport;

use serde_json::json;
use transport::{EmailMessage, EmailTransport};

use crate::config::Config;

pub async fn send_verification(
    transport: &dyn EmailTransport,
    _cfg: &Config,
    to: &str,
    verify_url: &str,
) -> Result<(), transport::EmailError> {
    let subject = "Verify your email";
    let text = templates::verify_text(verify_url);
    let html = templates::verify_html(verify_url);
    transport
        .send(EmailMessage {
            to: to.to_string(),
            subject: subject.to_string(),
            text_body: text,
            html_body: html,
            meta: json!({}),
        })
        .await
}

pub async fn send_password_reset(
    transport: &dyn EmailTransport,
    _cfg: &Config,
    to: &str,
    reset_url: &str,
) -> Result<(), transport::EmailError> {
    let subject = "Reset your password";
    let text = templates::reset_text(reset_url);
    let html = templates::reset_html(reset_url);
    transport
        .send(EmailMessage {
            to: to.to_string(),
            subject: subject.to_string(),
            text_body: text,
            html_body: html,
            meta: json!({}),
        })
        .await
}

pub async fn send_magic_link(
    transport: &dyn EmailTransport,
    _cfg: &Config,
    to: &str,
    magic_url: &str,
) -> Result<(), transport::EmailError> {
    let subject = "Your sign-in link";
    let text = templates::magic_text(magic_url);
    let html = templates::magic_html(magic_url);
    transport
        .send(EmailMessage {
            to: to.to_string(),
            subject: subject.to_string(),
            text_body: text,
            html_body: html,
            meta: json!({}),
        })
        .await
}
