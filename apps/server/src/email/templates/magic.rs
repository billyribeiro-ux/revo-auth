use super::escape_html;

pub fn magic_text(url: &str) -> String {
    format!("Sign in:\n{url}\n")
}

pub fn magic_html(url: &str) -> String {
    let u = escape_html(url);
    format!("<!DOCTYPE html><html><body><p><a href=\"{u}\">Sign in</a></p></body></html>")
}
