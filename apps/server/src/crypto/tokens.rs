use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ring::digest;
use ring::rand::{SecureRandom, SystemRandom};

use crate::crypto::hkdf;

pub fn random_token_32() -> Result<Vec<u8>, ring::error::Unspecified> {
    let rng = SystemRandom::new();
    let mut buf = [0u8; 32];
    rng.fill(&mut buf)?;
    Ok(buf.to_vec())
}

pub fn token_b64url(token: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(token)
}

pub fn hash_session_token(token: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, token).as_ref().to_vec()
}

#[derive(Debug, Clone, Copy)]
pub struct TokenCipherError;

pub struct TokenCipher {
    cipher: Aes256Gcm,
}

impl TokenCipher {
    pub fn from_master(master: &str) -> Result<Self, TokenCipherError> {
        let mut key = [0u8; 32];
        hkdf::derive_key(master.as_bytes(), b"revo-auth-salt", b"token-cipher-v1", &mut key)
            .map_err(|_| TokenCipherError)?;
        Ok(Self { cipher: Aes256Gcm::new_from_slice(&key).map_err(|_| TokenCipherError)? })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, TokenCipherError> {
        let mut nonce_bytes = [0u8; 12];
        SystemRandom::new().fill(&mut nonce_bytes).map_err(|_| TokenCipherError)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self.cipher.encrypt(nonce, plaintext).map_err(|_| TokenCipherError)?;
        let mut out = nonce_bytes.to_vec();
        out.extend_from_slice(&ct);
        Ok(out)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, TokenCipherError> {
        if ciphertext.len() < 12 {
            return Err(TokenCipherError);
        }
        let (n, ct) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(n);
        self.cipher.decrypt(nonce, ct).map_err(|_| TokenCipherError)
    }
}
