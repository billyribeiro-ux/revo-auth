mod invite;
mod magic;
mod reset;
mod verify;

pub use invite::{invite_html, invite_text};
pub use magic::{magic_html, magic_text};
pub use reset::{reset_html, reset_text};
pub use verify::{verify_html, verify_text};

/// HTML-escape attribute/text content for email templates. Delegates to
/// `html_escape` so behavior matches other templating layers.
pub(crate) fn escape_html(s: &str) -> String {
    html_escape::encode_safe(s).into_owned()
}
