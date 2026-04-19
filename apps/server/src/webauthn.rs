//! WebAuthn / passkey configuration (webauthn-rs).

use url::Url;
use webauthn_rs::prelude::*;

pub fn build_webauthn(rp_id: &str, origin: &str) -> Result<Webauthn, WebauthnError> {
    let origin_url: Url = origin.parse().map_err(|_| WebauthnError::Configuration)?;
    WebauthnBuilder::new(rp_id, &origin_url).and_then(|b| b.build())
}
