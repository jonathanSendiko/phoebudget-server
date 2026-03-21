use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::{Claims, get_keys, hash_password, verify_password};
use crate::error::AppError;
use crate::repository::{
    PocketRepository, RefreshTokenRepository, SettingsRepository, SubscriptionRepository,
    UserIdentityRepository, UserRepository,
};
use crate::schemas::{
    AuthResponse, LoginRequest, OAuthLoginRequest, RegisterRequest, UserIdentityRow, UserProfile,
};

use jsonwebtoken::{Header, encode};

#[async_trait]
pub trait AuthUserRepo: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<crate::schemas::User>, AppError>;
    async fn check_exists(&self, email: &str, username: &str) -> Result<bool, AppError>;
    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Uuid, AppError>;
    async fn create_oauth(&self, username: &str, email: &str) -> Result<Uuid, AppError>;
    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError>;
    async fn username_exists(&self, username: &str) -> Result<bool, AppError>;
}

#[async_trait]
pub trait AuthSettingsRepo: Send + Sync {
    async fn validate_currency(&self, code: &str) -> Result<bool, AppError>;
    async fn set_base_currency(&self, user_id: Uuid, currency: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait AuthPocketRepo: Send + Sync {
    async fn create_default_for_user(&self, user_id: Uuid) -> Result<Uuid, AppError>;
}

#[async_trait]
pub trait AuthRefreshTokenRepo: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<Uuid, AppError>;
    async fn find_by_hash_and_user(
        &self,
        token_hash: &str,
    ) -> Result<Option<crate::schemas::RefreshTokenRow>, AppError>;
    async fn rotate(&self, old_id: Uuid, new_hash: &str) -> Result<(), AppError>;
    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), AppError>;
}

#[async_trait]
pub trait AuthSubscriptionRepo: Send + Sync {
    async fn create_default(&self, user_id: Uuid) -> Result<Uuid, AppError>;
}

#[async_trait]
pub trait AuthUserIdentityRepo: Send + Sync {
    async fn find_by_provider_subject(
        &self,
        provider: &str,
        provider_subject: &str,
    ) -> Result<Option<UserIdentityRow>, AppError>;
    async fn create_identity(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_subject: &str,
        email: Option<&str>,
        email_verified: Option<bool>,
        name: Option<&str>,
        picture_url: Option<&str>,
    ) -> Result<Uuid, AppError>;
}

#[derive(Debug, Clone)]
pub struct OAuthClaims {
    pub provider: String,
    pub subject: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture_url: Option<String>,
}

#[async_trait]
pub trait OAuthIdTokenVerifier: Send + Sync {
    async fn verify(&self, id_token: &str, audience: &str) -> Result<OAuthClaims, AppError>;
}

pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &str) -> Result<String, AppError>;
    fn verify(&self, password: &str, password_hash: &str) -> Result<bool, AppError>;
}

pub struct DefaultPasswordHasher;

impl PasswordHasher for DefaultPasswordHasher {
    fn hash(&self, password: &str) -> Result<String, AppError> {
        hash_password(password)
    }

    fn verify(&self, password: &str, password_hash: &str) -> Result<bool, AppError> {
        verify_password(password, password_hash)
    }
}

#[async_trait]
impl AuthUserRepo for UserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<crate::schemas::User>, AppError> {
        self.find_by_email(email).await
    }

    async fn check_exists(&self, email: &str, username: &str) -> Result<bool, AppError> {
        self.check_exists(email, username).await
    }

    async fn create(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<Uuid, AppError> {
        self.create(username, email, password_hash).await
    }

    async fn create_oauth(&self, username: &str, email: &str) -> Result<Uuid, AppError> {
        self.create_oauth(username, email).await
    }

    async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        self.get_profile(user_id).await
    }

    async fn username_exists(&self, username: &str) -> Result<bool, AppError> {
        self.username_exists(username).await
    }
}

#[async_trait]
impl AuthSettingsRepo for SettingsRepository {
    async fn validate_currency(&self, code: &str) -> Result<bool, AppError> {
        self.validate_currency(code).await
    }

    async fn set_base_currency(&self, user_id: Uuid, currency: &str) -> Result<(), AppError> {
        self.set_base_currency(user_id, currency).await
    }
}

#[async_trait]
impl AuthPocketRepo for PocketRepository {
    async fn create_default_for_user(&self, user_id: Uuid) -> Result<Uuid, AppError> {
        self.create_default_for_user(user_id).await
    }
}

#[async_trait]
impl AuthRefreshTokenRepo for RefreshTokenRepository {
    async fn create(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<Uuid, AppError> {
        self.create(user_id, token_hash, expires_at).await
    }

    async fn find_by_hash_and_user(
        &self,
        token_hash: &str,
    ) -> Result<Option<crate::schemas::RefreshTokenRow>, AppError> {
        self.find_by_hash_and_user(token_hash).await
    }

    async fn rotate(&self, old_id: Uuid, new_hash: &str) -> Result<(), AppError> {
        self.rotate(old_id, new_hash).await
    }

    async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), AppError> {
        self.revoke_all_for_user(user_id).await
    }
}

#[async_trait]
impl AuthSubscriptionRepo for SubscriptionRepository {
    async fn create_default(&self, user_id: Uuid) -> Result<Uuid, AppError> {
        self.create_default(user_id).await
    }
}

#[async_trait]
impl AuthUserIdentityRepo for UserIdentityRepository {
    async fn find_by_provider_subject(
        &self,
        provider: &str,
        provider_subject: &str,
    ) -> Result<Option<UserIdentityRow>, AppError> {
        self.find_by_provider_subject(provider, provider_subject)
            .await
    }

    async fn create_identity(
        &self,
        user_id: Uuid,
        provider: &str,
        provider_subject: &str,
        email: Option<&str>,
        email_verified: Option<bool>,
        name: Option<&str>,
        picture_url: Option<&str>,
    ) -> Result<Uuid, AppError> {
        self.create_identity(
            user_id,
            provider,
            provider_subject,
            email,
            email_verified,
            name,
            picture_url,
        )
        .await
    }
}

pub trait TokenIssuer: Send + Sync {
    fn generate(&self, user_id: Uuid) -> Result<String, AppError>;
}

pub struct DefaultTokenIssuer;

impl TokenIssuer for DefaultTokenIssuer {
    fn generate(&self, user_id: Uuid) -> Result<String, AppError> {
        let claims = Claims {
            sub: user_id.to_string(),
            company: "Phoebudget".to_string(),
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp() as usize, // Reduced to 1 hour
        };

        encode(&Header::default(), &claims, &get_keys().encoding)
            .map_err(|_| AppError::InternalServerError("Token creation failed".to_string()))
    }
}

#[derive(Deserialize, Clone)]
struct GoogleJwks {
    keys: Vec<GoogleJwk>,
}

#[derive(Deserialize, Clone)]
struct GoogleJwk {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

#[derive(Clone)]
struct GoogleJwksCache {
    fetched_at: Instant,
    jwks: GoogleJwks,
}

impl Default for GoogleJwksCache {
    fn default() -> Self {
        Self {
            fetched_at: Instant::now() - Duration::from_secs(7200),
            jwks: GoogleJwks { keys: Vec::new() },
        }
    }
}

static GOOGLE_JWKS_CACHE: OnceLock<RwLock<GoogleJwksCache>> = OnceLock::new();

fn google_jwks_cache() -> &'static RwLock<GoogleJwksCache> {
    GOOGLE_JWKS_CACHE.get_or_init(|| RwLock::new(GoogleJwksCache::default()))
}

pub struct GoogleIdTokenVerifier {
    http_client: reqwest::Client,
    jwks_ttl: Duration,
}

impl GoogleIdTokenVerifier {
    pub fn new(http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            jwks_ttl: Duration::from_secs(3600),
        }
    }

    async fn get_jwks(&self) -> Result<GoogleJwks, AppError> {
        {
            let cache = google_jwks_cache().read().await;
            if !cache.jwks.keys.is_empty() && cache.fetched_at.elapsed() < self.jwks_ttl {
                return Ok(cache.jwks.clone());
            }
        }

        let mut cache = google_jwks_cache().write().await;
        if !cache.jwks.keys.is_empty() && cache.fetched_at.elapsed() < self.jwks_ttl {
            return Ok(cache.jwks.clone());
        }

        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v3/certs")
            .send()
            .await
            .map_err(|_| AppError::InternalServerError("Failed to fetch Google JWKS".to_string()))?
            .error_for_status()
            .map_err(|_| {
                AppError::InternalServerError("Failed to fetch Google JWKS".to_string())
            })?;

        let jwks: GoogleJwks = response.json().await.map_err(|_| {
            AppError::InternalServerError("Failed to parse Google JWKS".to_string())
        })?;

        cache.fetched_at = Instant::now();
        cache.jwks = jwks.clone();
        Ok(jwks)
    }
}

#[derive(Deserialize)]
struct GoogleIdTokenClaims {
    sub: String,
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
    aud: serde_json::Value,
    azp: Option<String>,
    iss: Option<String>,
}

#[async_trait]
impl OAuthIdTokenVerifier for GoogleIdTokenVerifier {
    async fn verify(&self, id_token: &str, audience: &str) -> Result<OAuthClaims, AppError> {
        let header = decode_header(id_token)
            .map_err(|_| AppError::AuthError("Invalid OAuth token header".to_string()))?;

        if header.alg != Algorithm::RS256 {
            return Err(AppError::AuthError(
                "Invalid OAuth token algorithm".to_string(),
            ));
        }

        let kid = header.kid.ok_or(AppError::AuthError(
            "Missing OAuth token key id".to_string(),
        ))?;

        let jwks = self.get_jwks().await?;
        let jwk = jwks
            .keys
            .iter()
            .find(|key| key.kid == kid)
            .ok_or(AppError::AuthError("Unknown OAuth token key".to_string()))?;

        if jwk.kty != "RSA" {
            return Err(AppError::AuthError(
                "Invalid OAuth token key type".to_string(),
            ));
        }

        let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
            .map_err(|_| AppError::AuthError("Invalid OAuth token key".to_string()))?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        validation.validate_aud = false;

        let token_data = decode::<GoogleIdTokenClaims>(id_token, &decoding_key, &validation)
            .map_err(|_| AppError::AuthError("Invalid OAuth token".to_string()))?;

        let issuer_ok = matches!(
            token_data.claims.iss.as_deref(),
            Some("accounts.google.com") | Some("https://accounts.google.com")
        );
        if !issuer_ok {
            return Err(AppError::AuthError(
                "Invalid OAuth token issuer".to_string(),
            ));
        }

        if !audience_matches(
            &token_data.claims.aud,
            audience,
            token_data.claims.azp.as_deref(),
        ) {
            return Err(AppError::AuthError(
                "Invalid OAuth token audience".to_string(),
            ));
        }

        Ok(OAuthClaims {
            provider: "google".to_string(),
            subject: token_data.claims.sub,
            email: token_data.claims.email,
            email_verified: token_data.claims.email_verified,
            name: token_data.claims.name,
            picture_url: token_data.claims.picture,
        })
    }
}

fn audience_matches(aud: &serde_json::Value, audience: &str, azp: Option<&str>) -> bool {
    match aud {
        serde_json::Value::String(value) => value == audience,
        serde_json::Value::Array(values) => {
            let has_aud = values
                .iter()
                .filter_map(|v| v.as_str())
                .any(|v| v == audience);
            if !has_aud {
                return false;
            }
            if let Some(azp_value) = azp {
                azp_value == audience
            } else {
                true
            }
        }
        _ => false,
    }
}

pub type AuthServiceImpl = AuthService<
    UserRepository,
    SettingsRepository,
    PocketRepository,
    RefreshTokenRepository,
    SubscriptionRepository,
    UserIdentityRepository,
    DefaultPasswordHasher,
    DefaultTokenIssuer,
    GoogleIdTokenVerifier,
>;

pub struct AuthService<URepo, SRepo, PRepo, RRepo, SubRepo, IRepo, Hasher, Issuer, Verifier> {
    user_repo: URepo,
    settings_repo: SRepo,
    pocket_repo: PRepo,
    refresh_token_repo: RRepo,
    subscription_repo: SubRepo,
    identity_repo: IRepo,
    password_hasher: Hasher,
    token_issuer: Issuer,
    oauth_verifier: Verifier,
}

impl<URepo, SRepo, PRepo, RRepo, SubRepo, IRepo, Hasher, Issuer, Verifier>
    AuthService<URepo, SRepo, PRepo, RRepo, SubRepo, IRepo, Hasher, Issuer, Verifier>
where
    URepo: AuthUserRepo,
    SRepo: AuthSettingsRepo,
    PRepo: AuthPocketRepo,
    RRepo: AuthRefreshTokenRepo,
    SubRepo: AuthSubscriptionRepo,
    IRepo: AuthUserIdentityRepo,
    Hasher: PasswordHasher,
    Issuer: TokenIssuer,
    Verifier: OAuthIdTokenVerifier,
{
    pub fn new(
        user_repo: URepo,
        settings_repo: SRepo,
        pocket_repo: PRepo,
        refresh_token_repo: RRepo,
        subscription_repo: SubRepo,
        identity_repo: IRepo,
        password_hasher: Hasher,
        token_issuer: Issuer,
        oauth_verifier: Verifier,
    ) -> Self {
        Self {
            user_repo,
            settings_repo,
            pocket_repo,
            refresh_token_repo,
            subscription_repo,
            identity_repo,
            password_hasher,
            token_issuer,
            oauth_verifier,
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

        let hashed = self.password_hasher.hash(&req.password)?;
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

        let password_hash = user.password_hash.as_deref().ok_or(AppError::AuthError(
            "Password login not enabled for this account".to_string(),
        ))?;

        if !self.password_hasher.verify(&req.password, password_hash)? {
            return Err(AppError::AuthError("Invalid credentials".to_string()));
        }

        let (token, refresh_token) = self.generate_tokens(user.id).await?;

        Ok(AuthResponse {
            token,
            refresh_token,
            message: "Login successful".to_string(),
        })
    }

    pub async fn oauth_login(&self, req: OAuthLoginRequest) -> Result<AuthResponse, AppError> {
        let provider = req.provider.to_lowercase();
        if provider != "google" {
            return Err(AppError::ValidationError(
                "Unsupported OAuth provider".to_string(),
            ));
        }

        let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|_| {
            AppError::InternalServerError("GOOGLE_CLIENT_ID must be set".to_string())
        })?;

        let claims = self
            .oauth_verifier
            .verify(&req.id_token, &client_id)
            .await?;
        let email = claims.email.ok_or(AppError::AuthError(
            "OAuth account missing email".to_string(),
        ))?;
        if !claims.email_verified.unwrap_or(false) {
            return Err(AppError::AuthError("OAuth email not verified".to_string()));
        }

        if let Some(identity) = self
            .identity_repo
            .find_by_provider_subject(&provider, &claims.subject)
            .await?
        {
            let (token, refresh_token) = self.generate_tokens(identity.user_id).await?;
            return Ok(AuthResponse {
                token,
                refresh_token,
                message: "Login successful".to_string(),
            });
        }

        if let Some(user) = self.user_repo.find_by_email(&email).await? {
            self.identity_repo
                .create_identity(
                    user.id,
                    &provider,
                    &claims.subject,
                    Some(&email),
                    claims.email_verified,
                    claims.name.as_deref(),
                    claims.picture_url.as_deref(),
                )
                .await?;

            let (token, refresh_token) = self.generate_tokens(user.id).await?;
            return Ok(AuthResponse {
                token,
                refresh_token,
                message: "Login successful".to_string(),
            });
        }

        let username = match req.username {
            Some(username) => {
                if self.user_repo.username_exists(&username).await? {
                    return Err(AppError::ValidationError(
                        "Username already taken".to_string(),
                    ));
                }
                username
            }
            None => self.generate_username(&email).await?,
        };

        let base_currency = req.base_currency.unwrap_or_else(|| "SGD".to_string());
        if !self.settings_repo.validate_currency(&base_currency).await? {
            return Err(AppError::ValidationError(format!(
                "Invalid currency code: {}",
                base_currency
            )));
        }

        let user_id = self.user_repo.create_oauth(&username, &email).await?;
        self.settings_repo
            .set_base_currency(user_id, &base_currency)
            .await?;
        self.pocket_repo.create_default_for_user(user_id).await?;
        self.subscription_repo.create_default(user_id).await?;
        self.identity_repo
            .create_identity(
                user_id,
                &provider,
                &claims.subject,
                Some(&email),
                claims.email_verified,
                claims.name.as_deref(),
                claims.picture_url.as_deref(),
            )
            .await?;

        let (token, refresh_token) = self.generate_tokens(user_id).await?;
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
        let access_token = self.token_issuer.generate(user_id)?;

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

    async fn generate_username(&self, email: &str) -> Result<String, AppError> {
        let local = email.split('@').next().unwrap_or("user");
        let mut base = local
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '_'
                }
            })
            .collect::<String>();
        base = base.trim_matches('_').to_string();
        if base.is_empty() {
            base = "user".to_string();
        }

        if !self.user_repo.username_exists(&base).await? {
            return Ok(base);
        }

        for suffix in 1..100 {
            let candidate = format!("{}{}", base, suffix);
            if !self.user_repo.username_exists(&candidate).await? {
                return Ok(candidate);
            }
        }

        Err(AppError::InternalServerError(
            "Unable to generate username".to_string(),
        ))
    }

    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfile, AppError> {
        self.user_repo.get_profile(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuthPocketRepo, AuthRefreshTokenRepo, AuthService, AuthSettingsRepo, AuthSubscriptionRepo,
        AuthUserIdentityRepo, AuthUserRepo, OAuthClaims, OAuthIdTokenVerifier, PasswordHasher,
        TokenIssuer,
    };
    use crate::error::AppError;
    use crate::schemas::{LoginRequest, OAuthLoginRequest, RefreshTokenRow, RegisterRequest, User};
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use sha2::{Digest, Sha256};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct MockUserRepo {
        state: Arc<Mutex<MockUserState>>,
    }

    struct MockUserState {
        exists: bool,
        user: Option<User>,
        created_user: Option<(String, String, String)>,
        profile_calls: usize,
    }

    impl Default for MockUserState {
        fn default() -> Self {
            Self {
                exists: false,
                user: None,
                created_user: None,
                profile_calls: 0,
            }
        }
    }

    #[async_trait]
    impl AuthUserRepo for MockUserRepo {
        async fn find_by_email(&self, _email: &str) -> Result<Option<User>, AppError> {
            Ok(self.state.lock().unwrap().user.clone())
        }

        async fn check_exists(&self, _email: &str, _username: &str) -> Result<bool, AppError> {
            Ok(self.state.lock().unwrap().exists)
        }

        async fn create(
            &self,
            username: &str,
            email: &str,
            password_hash: &str,
        ) -> Result<Uuid, AppError> {
            let mut state = self.state.lock().unwrap();
            state.created_user = Some((
                username.to_string(),
                email.to_string(),
                password_hash.to_string(),
            ));
            Ok(Uuid::new_v4())
        }

        async fn create_oauth(&self, username: &str, email: &str) -> Result<Uuid, AppError> {
            let mut state = self.state.lock().unwrap();
            state.created_user = Some((username.to_string(), email.to_string(), String::new()));
            Ok(Uuid::new_v4())
        }

        async fn get_profile(
            &self,
            _user_id: Uuid,
        ) -> Result<crate::schemas::UserProfile, AppError> {
            let mut state = self.state.lock().unwrap();
            state.profile_calls += 1;
            Err(AppError::InternalServerError("not used".to_string()))
        }

        async fn username_exists(&self, username: &str) -> Result<bool, AppError> {
            let state = self.state.lock().unwrap();
            if let Some((created_username, _, _)) = &state.created_user {
                return Ok(created_username == username);
            }
            Ok(false)
        }
    }

    #[derive(Clone)]
    struct MockSettingsRepo {
        valid_currency: bool,
        set_calls: Arc<Mutex<Vec<(Uuid, String)>>>,
    }

    #[async_trait]
    impl AuthSettingsRepo for MockSettingsRepo {
        async fn validate_currency(&self, _code: &str) -> Result<bool, AppError> {
            Ok(self.valid_currency)
        }

        async fn set_base_currency(&self, user_id: Uuid, currency: &str) -> Result<(), AppError> {
            self.set_calls
                .lock()
                .unwrap()
                .push((user_id, currency.to_string()));
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MockPocketRepo {
        calls: Arc<Mutex<Vec<Uuid>>>,
    }

    #[async_trait]
    impl AuthPocketRepo for MockPocketRepo {
        async fn create_default_for_user(&self, user_id: Uuid) -> Result<Uuid, AppError> {
            self.calls.lock().unwrap().push(user_id);
            Ok(Uuid::new_v4())
        }
    }

    #[derive(Clone, Default)]
    struct MockSubscriptionRepo {
        calls: Arc<Mutex<Vec<Uuid>>>,
    }

    #[async_trait]
    impl AuthSubscriptionRepo for MockSubscriptionRepo {
        async fn create_default(&self, user_id: Uuid) -> Result<Uuid, AppError> {
            self.calls.lock().unwrap().push(user_id);
            Ok(Uuid::new_v4())
        }
    }

    #[derive(Clone, Default)]
    struct MockIdentityRepo {
        state: Arc<Mutex<MockIdentityState>>,
    }

    struct MockIdentityState {
        by_provider_subject: HashMap<(String, String), crate::schemas::UserIdentityRow>,
        created: Vec<(Uuid, String, String)>,
    }

    impl Default for MockIdentityState {
        fn default() -> Self {
            Self {
                by_provider_subject: HashMap::new(),
                created: Vec::new(),
            }
        }
    }

    #[async_trait]
    impl AuthUserIdentityRepo for MockIdentityRepo {
        async fn find_by_provider_subject(
            &self,
            provider: &str,
            provider_subject: &str,
        ) -> Result<Option<crate::schemas::UserIdentityRow>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .by_provider_subject
                .get(&(provider.to_string(), provider_subject.to_string()))
                .cloned())
        }

        async fn create_identity(
            &self,
            user_id: Uuid,
            provider: &str,
            provider_subject: &str,
            _email: Option<&str>,
            _email_verified: Option<bool>,
            _name: Option<&str>,
            _picture_url: Option<&str>,
        ) -> Result<Uuid, AppError> {
            self.state.lock().unwrap().created.push((
                user_id,
                provider.to_string(),
                provider_subject.to_string(),
            ));
            Ok(Uuid::new_v4())
        }
    }

    #[derive(Clone, Default)]
    struct MockRefreshTokenRepo {
        state: Arc<Mutex<MockRefreshTokenState>>,
    }

    struct MockRefreshTokenState {
        by_hash: HashMap<String, RefreshTokenRow>,
        created: Vec<(Uuid, String)>,
        rotated: Vec<(Uuid, String)>,
        revoked: Vec<Uuid>,
    }

    impl Default for MockRefreshTokenState {
        fn default() -> Self {
            Self {
                by_hash: HashMap::new(),
                created: Vec::new(),
                rotated: Vec::new(),
                revoked: Vec::new(),
            }
        }
    }

    #[async_trait]
    impl AuthRefreshTokenRepo for MockRefreshTokenRepo {
        async fn create(
            &self,
            user_id: Uuid,
            token_hash: &str,
            _expires_at: DateTime<Utc>,
        ) -> Result<Uuid, AppError> {
            self.state
                .lock()
                .unwrap()
                .created
                .push((user_id, token_hash.to_string()));
            Ok(Uuid::new_v4())
        }

        async fn find_by_hash_and_user(
            &self,
            token_hash: &str,
        ) -> Result<Option<RefreshTokenRow>, AppError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .by_hash
                .get(token_hash)
                .map(clone_refresh_token_row_ref))
        }

        async fn rotate(&self, old_id: Uuid, new_hash: &str) -> Result<(), AppError> {
            self.state
                .lock()
                .unwrap()
                .rotated
                .push((old_id, new_hash.to_string()));
            Ok(())
        }

        async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<(), AppError> {
            self.state.lock().unwrap().revoked.push(user_id);
            Ok(())
        }
    }

    fn clone_refresh_token_row_ref(row: &RefreshTokenRow) -> RefreshTokenRow {
        RefreshTokenRow {
            id: row.id,
            user_id: row.user_id,
            token_hash: row.token_hash.clone(),
            expires_at: row.expires_at,
            created_at: row.created_at,
            replaced_by: row.replaced_by.clone(),
            is_revoked: row.is_revoked,
        }
    }

    #[derive(Clone)]
    struct MockPasswordHasher {
        hash_value: String,
        verify_ok: bool,
    }

    impl PasswordHasher for MockPasswordHasher {
        fn hash(&self, _password: &str) -> Result<String, AppError> {
            Ok(self.hash_value.clone())
        }

        fn verify(&self, _password: &str, _password_hash: &str) -> Result<bool, AppError> {
            Ok(self.verify_ok)
        }
    }

    #[derive(Clone)]
    struct MockTokenIssuer {
        token: String,
    }

    impl TokenIssuer for MockTokenIssuer {
        fn generate(&self, _user_id: Uuid) -> Result<String, AppError> {
            Ok(self.token.clone())
        }
    }

    #[derive(Clone)]
    struct MockOauthVerifier {
        claims: OAuthClaims,
    }

    #[async_trait]
    impl OAuthIdTokenVerifier for MockOauthVerifier {
        async fn verify(&self, _id_token: &str, _audience: &str) -> Result<OAuthClaims, AppError> {
            Ok(self.claims.clone())
        }
    }

    fn make_service(
        user_repo: MockUserRepo,
        settings_repo: MockSettingsRepo,
        pocket_repo: MockPocketRepo,
        refresh_token_repo: MockRefreshTokenRepo,
        subscription_repo: MockSubscriptionRepo,
        identity_repo: MockIdentityRepo,
        hasher: MockPasswordHasher,
        issuer: MockTokenIssuer,
        verifier: MockOauthVerifier,
    ) -> AuthService<
        MockUserRepo,
        MockSettingsRepo,
        MockPocketRepo,
        MockRefreshTokenRepo,
        MockSubscriptionRepo,
        MockIdentityRepo,
        MockPasswordHasher,
        MockTokenIssuer,
        MockOauthVerifier,
    > {
        AuthService::new(
            user_repo,
            settings_repo,
            pocket_repo,
            refresh_token_repo,
            subscription_repo,
            identity_repo,
            hasher,
            issuer,
            verifier,
        )
    }

    fn sample_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password_hash: Some("hashed".to_string()),
            created_at: None,
        }
    }

    fn default_oauth_verifier() -> MockOauthVerifier {
        MockOauthVerifier {
            claims: OAuthClaims {
                provider: "google".to_string(),
                subject: "subject-1".to_string(),
                email: Some("alice@example.com".to_string()),
                email_verified: Some(true),
                name: Some("Alice".to_string()),
                picture_url: None,
            },
        }
    }

    #[tokio::test]
    async fn register_rejects_existing_user() {
        let user_repo = MockUserRepo {
            state: Arc::new(Mutex::new(MockUserState {
                exists: true,
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            valid_currency: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let service = make_service(
            user_repo,
            settings_repo,
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "pass".to_string(),
            base_currency: "USD".to_string(),
        };

        let err = service.register(req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "User with this email or username already exists")
        );
    }

    #[tokio::test]
    async fn register_rejects_invalid_currency() {
        let user_repo = MockUserRepo::default();
        let settings_repo = MockSettingsRepo {
            valid_currency: false,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let service = make_service(
            user_repo,
            settings_repo,
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "pass".to_string(),
            base_currency: "BAD".to_string(),
        };

        let err = service.register(req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Invalid currency code: BAD")
        );
    }

    #[tokio::test]
    async fn register_creates_defaults_and_tokens() {
        let user_repo = MockUserRepo::default();
        let settings_repo = MockSettingsRepo {
            valid_currency: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let pocket_repo = MockPocketRepo::default();
        let refresh_repo = MockRefreshTokenRepo::default();
        let subscription_repo = MockSubscriptionRepo::default();
        let identity_repo = MockIdentityRepo::default();
        let hasher = MockPasswordHasher {
            hash_value: "hashed-pass".to_string(),
            verify_ok: true,
        };
        let service = make_service(
            user_repo.clone(),
            settings_repo.clone(),
            pocket_repo.clone(),
            refresh_repo.clone(),
            subscription_repo.clone(),
            identity_repo.clone(),
            hasher,
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = RegisterRequest {
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
            password: "pass".to_string(),
            base_currency: "USD".to_string(),
        };

        let resp = service.register(req).await.unwrap();
        assert!(!resp.token.is_empty());
        assert!(!resp.refresh_token.is_empty());
        assert_eq!(resp.message, "Registration successful");

        let created = user_repo.state.lock().unwrap().created_user.clone();
        assert!(created.is_some());
        let (_, _, password_hash) = created.unwrap();
        assert_eq!(password_hash, "hashed-pass");

        assert_eq!(pocket_repo.calls.lock().unwrap().len(), 1);
        assert_eq!(subscription_repo.calls.lock().unwrap().len(), 1);
        assert_eq!(refresh_repo.state.lock().unwrap().created.len(), 1);
        assert_eq!(settings_repo.set_calls.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn login_rejects_unknown_user() {
        let user_repo = MockUserRepo::default();
        let settings_repo = MockSettingsRepo {
            valid_currency: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let service = make_service(
            user_repo,
            settings_repo,
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = LoginRequest {
            email: "missing@example.com".to_string(),
            password: "pass".to_string(),
        };
        let err = service.login(req).await.unwrap_err();
        assert!(matches!(err, AppError::AuthError(msg) if msg == "Invalid credentials"));
    }

    #[tokio::test]
    async fn login_rejects_invalid_password() {
        let user_repo = MockUserRepo {
            state: Arc::new(Mutex::new(MockUserState {
                user: Some(sample_user()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            valid_currency: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let service = make_service(
            user_repo,
            settings_repo,
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: false,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = LoginRequest {
            email: "alice@example.com".to_string(),
            password: "wrong".to_string(),
        };
        let err = service.login(req).await.unwrap_err();
        assert!(matches!(err, AppError::AuthError(msg) if msg == "Invalid credentials"));
    }

    #[tokio::test]
    async fn login_success_returns_tokens() {
        let user_repo = MockUserRepo {
            state: Arc::new(Mutex::new(MockUserState {
                user: Some(sample_user()),
                ..Default::default()
            })),
        };
        let settings_repo = MockSettingsRepo {
            valid_currency: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let refresh_repo = MockRefreshTokenRepo::default();
        let service = make_service(
            user_repo,
            settings_repo,
            MockPocketRepo::default(),
            refresh_repo.clone(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = LoginRequest {
            email: "alice@example.com".to_string(),
            password: "pass".to_string(),
        };
        let resp = service.login(req).await.unwrap();
        assert_eq!(resp.message, "Login successful");
        assert!(!resp.token.is_empty());
        assert!(!resp.refresh_token.is_empty());
        assert_eq!(refresh_repo.state.lock().unwrap().created.len(), 1);
    }

    #[tokio::test]
    async fn oauth_login_rejects_invalid_provider() {
        unsafe { std::env::set_var("GOOGLE_CLIENT_ID", "client-id") };
        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = OAuthLoginRequest {
            provider: "github".to_string(),
            id_token: "token".to_string(),
            username: None,
            base_currency: None,
        };

        let err = service.oauth_login(req).await.unwrap_err();
        assert!(
            matches!(err, AppError::ValidationError(msg) if msg == "Unsupported OAuth provider")
        );
    }

    #[tokio::test]
    async fn oauth_login_rejects_unverified_email() {
        unsafe { std::env::set_var("GOOGLE_CLIENT_ID", "client-id") };
        let verifier = MockOauthVerifier {
            claims: OAuthClaims {
                provider: "google".to_string(),
                subject: "subject-1".to_string(),
                email: Some("alice@example.com".to_string()),
                email_verified: Some(false),
                name: None,
                picture_url: None,
            },
        };
        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            verifier,
        );

        let req = OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "token".to_string(),
            username: None,
            base_currency: None,
        };

        let err = service.oauth_login(req).await.unwrap_err();
        assert!(matches!(err, AppError::AuthError(msg) if msg == "OAuth email not verified"));
    }

    #[tokio::test]
    async fn oauth_login_returns_tokens_for_existing_identity() {
        unsafe { std::env::set_var("GOOGLE_CLIENT_ID", "client-id") };
        let identity_repo = MockIdentityRepo::default();
        let identity = crate::schemas::UserIdentityRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            provider: "google".to_string(),
            provider_subject: "subject-1".to_string(),
            email: Some("alice@example.com".to_string()),
            email_verified: Some(true),
            name: None,
            picture_url: None,
            created_at: None,
            updated_at: None,
        };
        identity_repo
            .state
            .lock()
            .unwrap()
            .by_provider_subject
            .insert(
                ("google".to_string(), "subject-1".to_string()),
                identity.clone(),
            );

        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            identity_repo,
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "token".to_string(),
            username: None,
            base_currency: None,
        };

        let resp = service.oauth_login(req).await.unwrap();
        assert_eq!(resp.message, "Login successful");
        assert!(!resp.token.is_empty());
        assert!(!resp.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn oauth_login_links_existing_user_by_email() {
        unsafe { std::env::set_var("GOOGLE_CLIENT_ID", "client-id") };
        let user_repo = MockUserRepo {
            state: Arc::new(Mutex::new(MockUserState {
                user: Some(sample_user()),
                ..Default::default()
            })),
        };
        let identity_repo = MockIdentityRepo::default();
        let service = make_service(
            user_repo,
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            identity_repo.clone(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "token".to_string(),
            username: None,
            base_currency: None,
        };

        let resp = service.oauth_login(req).await.unwrap();
        assert_eq!(resp.message, "Login successful");
        assert_eq!(identity_repo.state.lock().unwrap().created.len(), 1);
    }

    #[tokio::test]
    async fn oauth_login_creates_new_user() {
        unsafe { std::env::set_var("GOOGLE_CLIENT_ID", "client-id") };
        let user_repo = MockUserRepo::default();
        let settings_repo = MockSettingsRepo {
            valid_currency: true,
            set_calls: Arc::new(Mutex::new(Vec::new())),
        };
        let pocket_repo = MockPocketRepo::default();
        let refresh_repo = MockRefreshTokenRepo::default();
        let subscription_repo = MockSubscriptionRepo::default();
        let identity_repo = MockIdentityRepo::default();

        let service = make_service(
            user_repo.clone(),
            settings_repo.clone(),
            pocket_repo.clone(),
            refresh_repo.clone(),
            subscription_repo.clone(),
            identity_repo.clone(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let req = OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "token".to_string(),
            username: None,
            base_currency: Some("USD".to_string()),
        };

        let resp = service.oauth_login(req).await.unwrap();
        assert_eq!(resp.message, "Login successful");
        assert!(user_repo.state.lock().unwrap().created_user.is_some());
        assert_eq!(pocket_repo.calls.lock().unwrap().len(), 1);
        assert_eq!(subscription_repo.calls.lock().unwrap().len(), 1);
        assert_eq!(settings_repo.set_calls.lock().unwrap().len(), 1);
        assert_eq!(identity_repo.state.lock().unwrap().created.len(), 1);
        assert_eq!(refresh_repo.state.lock().unwrap().created.len(), 1);
    }

    #[tokio::test]
    async fn refresh_access_rejects_invalid_token() {
        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            MockRefreshTokenRepo::default(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let err = service.refresh_access("bad").await.unwrap_err();
        assert!(matches!(err, AppError::AuthError(msg) if msg == "Invalid refresh token"));
    }

    #[tokio::test]
    async fn refresh_access_rejects_revoked_token() {
        let token = "token";
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            hex::encode(hasher.finalize())
        };
        let row = RefreshTokenRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: hash.clone(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: None,
            replaced_by: None,
            is_revoked: Some(true),
        };
        let mut refresh_state = MockRefreshTokenState::default();
        refresh_state.by_hash.insert(hash, row);
        let refresh_repo = MockRefreshTokenRepo {
            state: Arc::new(Mutex::new(refresh_state)),
        };
        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            refresh_repo,
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let err = service.refresh_access(token).await.unwrap_err();
        assert!(matches!(err, AppError::AuthError(msg) if msg == "Token revoked"));
    }

    #[tokio::test]
    async fn refresh_access_detects_reuse_and_revokes() {
        let token = "token";
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            hex::encode(hasher.finalize())
        };
        let user_id = Uuid::new_v4();
        let row = RefreshTokenRow {
            id: Uuid::new_v4(),
            user_id,
            token_hash: hash.clone(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: None,
            replaced_by: Some("repl".to_string()),
            is_revoked: Some(false),
        };
        let mut refresh_state = MockRefreshTokenState::default();
        refresh_state.by_hash.insert(hash, row);
        let refresh_repo = MockRefreshTokenRepo {
            state: Arc::new(Mutex::new(refresh_state)),
        };
        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            refresh_repo.clone(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let err = service.refresh_access(token).await.unwrap_err();
        assert!(
            matches!(err, AppError::AuthError(msg) if msg == "Security alert: Token reuse detected")
        );
        assert_eq!(refresh_repo.state.lock().unwrap().revoked, vec![user_id]);
    }

    #[tokio::test]
    async fn refresh_access_rejects_expired_token() {
        let token = "token";
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            hex::encode(hasher.finalize())
        };
        let row = RefreshTokenRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: hash.clone(),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            created_at: None,
            replaced_by: None,
            is_revoked: Some(false),
        };
        let mut refresh_state = MockRefreshTokenState::default();
        refresh_state.by_hash.insert(hash, row);
        let refresh_repo = MockRefreshTokenRepo {
            state: Arc::new(Mutex::new(refresh_state)),
        };
        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            refresh_repo,
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let err = service.refresh_access(token).await.unwrap_err();
        assert!(matches!(err, AppError::AuthError(msg) if msg == "Token expired"));
    }

    #[tokio::test]
    async fn refresh_access_rotates_token_on_success() {
        let token = "token";
        let hash = {
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            hex::encode(hasher.finalize())
        };
        let row = RefreshTokenRow {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: hash.clone(),
            expires_at: Utc::now() + chrono::Duration::hours(1),
            created_at: None,
            replaced_by: None,
            is_revoked: Some(false),
        };
        let mut refresh_state = MockRefreshTokenState::default();
        refresh_state.by_hash.insert(hash, row);
        let refresh_repo = MockRefreshTokenRepo {
            state: Arc::new(Mutex::new(refresh_state)),
        };
        let service = make_service(
            MockUserRepo::default(),
            MockSettingsRepo {
                valid_currency: true,
                set_calls: Arc::new(Mutex::new(Vec::new())),
            },
            MockPocketRepo::default(),
            refresh_repo.clone(),
            MockSubscriptionRepo::default(),
            MockIdentityRepo::default(),
            MockPasswordHasher {
                hash_value: "hash".to_string(),
                verify_ok: true,
            },
            MockTokenIssuer {
                token: "token".to_string(),
            },
            default_oauth_verifier(),
        );

        let resp = service.refresh_access(token).await.unwrap();
        assert_eq!(resp.message, "Token refreshed");
        assert!(!resp.token.is_empty());
        assert!(!resp.refresh_token.is_empty());

        let rotated = refresh_repo.state.lock().unwrap().rotated.clone();
        assert_eq!(rotated.len(), 1);
        let (_old_id, new_hash) = &rotated[0];
        assert_eq!(new_hash.len(), 64);
    }
}
