use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub aud: Uuid,
    pub iss: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn sign_es256(
    claims: &Claims,
    issuer: &str,
    private_pem: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let mut c: Claims = claims.clone();
    c.iss = issuer.to_string();
    let key = EncodingKey::from_ec_pem(private_pem.as_bytes())?;
    let mut header = Header::new(Algorithm::ES256);
    header.typ = Some("JWT".into());
    encode(&header, &c, &key)
}

pub fn verify_es256(
    token: &str,
    issuer: &str,
    audience: Uuid,
    public_pem: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_ec_pem(public_pem.as_bytes())?;
    let mut val = Validation::new(Algorithm::ES256);
    val.set_audience(&[audience.to_string()]);
    val.validate_exp = true;
    val.set_issuer(&[issuer]);
    let data = decode::<Claims>(token, &key, &val)?;
    Ok(data.claims)
}
