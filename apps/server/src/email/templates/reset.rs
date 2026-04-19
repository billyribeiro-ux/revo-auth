use super::escape_html;

pub fn reset_text(url: &str) -> String {
    format!("Reset your password:\n{url}\n")
}

pub fn reset_html(url: &str) -> String {
    let u = escape_html(url);
    format!("<!DOCTYPE html><html><body><p><a href=\"{u}\">Reset password</a></p></body></html>")
}
