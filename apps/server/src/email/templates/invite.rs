use super::escape_html;

pub fn invite_text(url: &str, org_name: &str) -> String {
    format!("You have been invited to join {org_name}:\n{url}\n")
}

pub fn invite_html(url: &str, org_name: &str) -> String {
    let u = escape_html(url);
    let n = escape_html(org_name);
    format!("<!DOCTYPE html><html><body><p>You have been invited to join {n}.</p><p><a href=\"{u}\">Accept invitation</a></p></body></html>")
}
