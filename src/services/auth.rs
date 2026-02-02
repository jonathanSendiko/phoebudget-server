use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::{Claims, get_keys, hash_password, verify_password};
use crate::error::AppError;
use crate::repository::{
    PocketRepository, RefreshTokenRepository, SettingsRepository, SubscriptionRepository,
    UserRepository,
};
use crate::schemas::{AuthResponse, LoginRequest, RegisterRequest, UserProfile};

use jsonwebtoken::{Header, encode};

pub struct AuthService {
    user_repo: UserRepository,
    settings_repo: SettingsRepository,
    pocket_repo: PocketRepository,
    refresh_token_repo: RefreshTokenRepository,
    subscription_repo: SubscriptionRepository,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        settings_repo: SettingsRepository,
        pocket_repo: PocketRepository,
        refresh_token_repo: RefreshTokenRepository,
        subscription_repo: SubscriptionRepository,
    ) -> Self {
        Self {
            user_repo,
            settings_repo,
            pocket_repo,
            refresh_token_repo,
            subscription_repo,
        }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, AppError> {
        if self
            .user_repo
            .check_exists(&req.email, &req.username)
            .await?
        {
            return Err(AppError::ValidationError(
                "User with this email or username already exists".to_string(),
            ));
        }

        if !self
            .settings_repo
            .validate_currency(&req.base_currency)
            .await?
        {
            return Err(AppError::ValidationError(format!(
                "Invalid currency code: {}",
                req.base_currency
            )));
        }

        let hashed = hash_password(&req.password)?;
        let user_id = self
            .user_repo
            .create(&req.username, &req.email, &hashed)
            .await?;

        self.settings_repo
            .set_base_currency(user_id, &req.base_currency)
            .await?;

        // Create default pocket for the new user
        self.pocket_repo.create_default_for_user(user_id).await?;

        // Create default free subscription for the new user
        self.subscription_repo.create_default(user_id).await?;

        // Auto-login (generate token)
        let (token, refresh_token) = self.generate_tokens(user_id).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            message: "Registration successful".to_string(),
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, AppError> {
        let user = self
            .user_repo
            .find_by_email(&req.email)
            .await?
            .ok_or(AppError::AuthError("Invalid credentials".to_string()))?;

        if !verify_password(&req.password, &user.password_hash)? {
            return Err(AppError::AuthError("Invalid credentials".to_string()));
        }

        let (token, refresh_token) = self.generate_tokens(user.id).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            message: "Login successful".to_string(),
        })
    }

    pub async fn refresh_access(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        // 1. Hash the incoming token
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        let hash = hex::encode(hasher.finalize());

        // 2. Find in DB
        let token_row = self
            .refresh_token_repo
            .find_by_hash_and_user(&hash)
            .await?
            .ok_or(AppError::AuthError("Invalid refresh token".to_string()))?;

        // 3. Security checks
        if token_row.is_revoked.unwrap_or(false) {
            // Already revoked explicitly
            return Err(AppError::AuthError("Token revoked".to_string()));
        }

        if let Some(_replacement) = token_row.replaced_by {
            // REUSE DETECTED!
            // This token was already rotated. Someone is trying to use an old token.
            // Revoke EVERYTHING for this user.
            tracing::warn!(
                "Refresh token reuse detected for user {}. Revoking all sessions.",
                token_row.user_id
            );
            self.refresh_token_repo
                .revoke_all_for_user(token_row.user_id)
                .await?;
            return Err(AppError::AuthError(
                "Security alert: Token reuse detected".to_string(),
            ));
        }

        if token_row.expires_at < Utc::now() {
            return Err(AppError::AuthError("Token expired".to_string()));
        }

        // 4. Rotate: Generate new pair, mark old as replaced
        let (new_access_token, new_refresh_token) = self.generate_tokens(token_row.user_id).await?;

        // Calculate hash of new token to link
        let mut new_hasher = Sha256::new();
        new_hasher.update(new_refresh_token.as_bytes());
        let new_hash = hex::encode(new_hasher.finalize());

        self.refresh_token_repo
            .rotate(token_row.id, &new_hash)
            .await?;

        Ok(AuthResponse {
            token: new_access_token,
            refresh_token: new_refresh_token,
            message: "Token refreshed".to_string(),
        })
    }

    async fn generate_tokens(&self, user_id: Uuid) -> Result<(String, String), AppError> {
        // JWT
        let access_token = self.generate_jwt(user_id)?;

        // Refresh Token (64 char hex string from 2 UUIDs)
        let refresh_token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());

        // Hash it
        let mut hasher = Sha256::new();
        hasher.update(refresh_token.as_bytes());
        let hash = hex::encode(hasher.finalize());

        // Save to DB (expires in 7 days)
        let expires_at = Utc::now() + chrono::Duration::days(7);
        self.refresh_token_repo
            .create(user_id, &hash, expires_at)
            .await?;

        Ok((access_token, refresh_token))
    }

    fn generate_jwt(&self, user_id: Uuid) -> Result<String, AppError> {
        let claims = Claims {
            sub: user_id.to_string(),
            company: "Phoebudget".to_string(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize, // Reduced to 1 hour
        };

        encode(&Header::default(), &claims, &get_keys().encoding)
            .map_err(|_| AppError::InternalServerError("Token creation failed".to_string()))
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        self.user_repo.get_profile(user_id).await
    }
}
