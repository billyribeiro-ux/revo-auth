mod invite;
mod magic;
mod reset;
mod verify;

pub use invite::{invite_html, invite_text};
pub use magic::{magic_html, magic_text};
pub use reset::{reset_html, reset_text};
pub use verify::{verify_html, verify_text};

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
