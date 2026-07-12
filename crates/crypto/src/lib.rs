use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, ParamsBuilder, Version};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const KDF_NAME: &str = "argon2id";
pub const CIPHER_NAME: &str = "aes-256-gcm";
pub const NONCE_LEN: usize = 12;
pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid password")]
    InvalidPassword,
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DerivedKey([u8; KEY_LEN]);

impl DerivedKey {
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::fill(&mut salt);
    salt
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<DerivedKey, CryptoError> {
    let params = argon2_params()?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
    Ok(DerivedKey(key))
}

pub fn encrypt(plaintext: &[u8], key: &[u8; KEY_LEN]) -> Result<(Vec<u8>, Vec<u8>), CryptoError> {
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::fill(&mut nonce_bytes);
    let nonce = Nonce::try_from(&nonce_bytes[..])
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    Ok((nonce_bytes.to_vec(), ciphertext))
}

pub fn decrypt(
    nonce: &[u8],
    ciphertext: &[u8],
    key: &[u8; KEY_LEN],
) -> Result<Vec<u8>, CryptoError> {
    if nonce.len() != NONCE_LEN {
        return Err(CryptoError::DecryptionFailed("invalid nonce length".into()));
    }
    let cipher =
        Aes256Gcm::new_from_slice(key).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    let nonce =
        Nonce::try_from(&nonce[..]).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| CryptoError::InvalidPassword)
}

fn argon2_params() -> Result<Params, CryptoError> {
    ParamsBuilder::new()
        .m_cost(19_456)
        .t_cost(2)
        .p_cost(1)
        .build()
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encrypt_decrypt() {
        let salt = generate_salt();
        let key = derive_key("test-password", &salt).unwrap();
        let plaintext = b"secret wallet payload";
        let (nonce, ciphertext) = encrypt(plaintext, key.as_bytes()).unwrap();
        let decrypted = decrypt(&nonce, &ciphertext, key.as_bytes()).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_password_fails_decrypt() {
        let salt = generate_salt();
        let key = derive_key("correct", &salt).unwrap();
        let wrong = derive_key("wrong", &salt).unwrap();
        let (nonce, ciphertext) = encrypt(b"data", key.as_bytes()).unwrap();
        assert!(decrypt(&nonce, &ciphertext, wrong.as_bytes()).is_err());
    }
}
