use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use once_cell::sync::Lazy;
use std::time::Instant;

use crate::error::ApiError;

const M_COST: u32 = 65536;
const T_COST: u32 = 3;
const P_COST: u32 = 1;

static COMMON: Lazy<Vec<&'static str>> = Lazy::new(|| {
    include_str!("../resources/common_passwords.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
});

pub fn hash_password(password: &str) -> Result<String, ApiError> {
    validate_password_strength(password)?;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(M_COST, T_COST, P_COST, None).map_err(|_| ApiError::Internal)?,
    );
    let hash = argon2.hash_password(password.as_bytes(), &salt).map_err(|_| ApiError::Internal)?;
    Ok(hash.to_string())
}

/// Argon2id for non-user secrets (e.g. app API keys). Skips password policy.
pub fn hash_app_secret(secret: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(M_COST, T_COST, P_COST, None).map_err(|_| ApiError::Internal)?,
    );
    let hash = argon2.hash_password(secret.as_bytes(), &salt).map_err(|_| ApiError::Internal)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, ph: &str) -> Result<bool, ApiError> {
    let parsed = PasswordHash::new(ph).map_err(|_| ApiError::InvalidCredentials)?;
    let argon2 = Argon2::default();
    Ok(argon2.verify_password(password.as_bytes(), &parsed).is_ok())
}

pub fn needs_rehash(ph: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(ph) else {
        return true;
    };
    let m: u32 = parsed.params.get_decimal("m").unwrap_or(0);
    let t: u32 = parsed.params.get_decimal("t").unwrap_or(0);
    let p: u32 = parsed.params.get_decimal("p").unwrap_or(0);
    m < M_COST || t < T_COST || p < P_COST
}

pub fn validate_password_strength(password: &str) -> Result<(), ApiError> {
    if password.len() < 12 {
        return Err(ApiError::Validation("password must be at least 12 characters".into()));
    }
    let lower = password.to_ascii_lowercase();
    if COMMON.iter().any(|c| lower == *c) {
        return Err(ApiError::Validation("password is too common; choose a stronger one".into()));
    }
    Ok(())
}

pub async fn constant_time_delay_miss() {
    let ms = 50 + (rand::random::<u8>() % 100) as u64;
    tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
}

pub fn verify_start() -> Instant {
    Instant::now()
}

pub async fn pad_verify_elapsed(start: Instant) {
    let elapsed = start.elapsed();
    let min = tokio::time::Duration::from_millis(200);
    if elapsed < min {
        tokio::time::sleep(min - elapsed).await;
    }
}
