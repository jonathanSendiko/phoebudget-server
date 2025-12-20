use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

use crate::error::AppError;

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub static KEYS: OnceLock<Keys> = OnceLock::new();

pub fn get_keys() -> &'static Keys {
    KEYS.get_or_init(|| {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        Keys::new(secret.as_bytes())
    })
}

// --- Claims ---
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub company: String,
    pub exp: usize,
}

// --- Password Utils ---

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| {
            eprintln!("Hashing error: {}", e);
            AppError::InternalServerError("Password hashing failed".to_string())
        })
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|e| {
        eprintln!("Hash parsing error: {}", e);
        AppError::InternalServerError("Invalid password hash in DB".to_string())
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub struct UserId(pub Uuid);
impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::AuthError("Missing or invalid token".to_string()))?;

        // Decode the token
        let token_data =
            decode::<Claims>(bearer.token(), &get_keys().decoding, &Validation::default())
                .map_err(|e| {
                    eprintln!("JWT Decode Error: {}", e);
                    AppError::AuthError("Invalid token".to_string())
                })?;

        // Parse UUID
        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::AuthError("Invalid user ID in token".to_string()))?;

        Ok(UserId(user_id))
    }
}
