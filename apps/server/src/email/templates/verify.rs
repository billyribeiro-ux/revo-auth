use super::escape_html;

pub fn verify_text(url: &str) -> String {
    format!("Verify your email by opening:\n{url}\n")
}

pub fn verify_html(url: &str) -> String {
    let u = escape_html(url);
    format!("<!DOCTYPE html><html><body><p><a href=\"{u}\">Verify email</a></p><p>{u}</p></body></html>")
}
