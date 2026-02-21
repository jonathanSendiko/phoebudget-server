use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use phoebudget::error::AppError;
use phoebudget::schemas::{OAuthLoginRequest, RefreshTokenRow, User, UserIdentityRow};
use phoebudget::services::{
    AuthPocketRepo, AuthRefreshTokenRepo, AuthService, AuthSettingsRepo, AuthSubscriptionRepo,
    AuthUserIdentityRepo, AuthUserRepo, OAuthClaims, OAuthIdTokenVerifier, PasswordHasher,
    TokenIssuer,
};

#[derive(Clone, Default)]
struct MockUserRepo {
    state: Arc<Mutex<MockUserState>>,
}

#[derive(Default)]
struct MockUserState {
    user: Option<User>,
    created_user: Option<(String, String)>,
    usernames: HashMap<String, bool>,
}

#[async_trait]
impl AuthUserRepo for MockUserRepo {
    async fn find_by_email(&self, _email: &str) -> Result<Option<User>, AppError> {
        Ok(self.state.lock().unwrap().user.clone())
    }

    async fn check_exists(&self, _email: &str, _username: &str) -> Result<bool, AppError> {
        Ok(false)
    }

    async fn create(
        &self,
        _username: &str,
        _email: &str,
        _password_hash: &str,
    ) -> Result<Uuid, AppError> {
        Err(AppError::InternalServerError("not used".to_string()))
    }

    async fn create_oauth(&self, username: &str, email: &str) -> Result<Uuid, AppError> {
        let mut state = self.state.lock().unwrap();
        state.created_user = Some((username.to_string(), email.to_string()));
        state.usernames.insert(username.to_string(), true);
        Ok(Uuid::new_v4())
    }

    async fn get_profile(
        &self,
        _user_id: Uuid,
    ) -> Result<phoebudget::schemas::UserProfile, AppError> {
        Err(AppError::InternalServerError("not used".to_string()))
    }

    async fn username_exists(&self, username: &str) -> Result<bool, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .usernames
            .get(username)
            .copied()
            .unwrap_or(false))
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
struct MockRefreshTokenRepo {
    state: Arc<Mutex<MockRefreshTokenState>>,
}

#[derive(Default)]
struct MockRefreshTokenState {
    created: Vec<(Uuid, String)>,
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
        _token_hash: &str,
    ) -> Result<Option<RefreshTokenRow>, AppError> {
        Ok(None)
    }

    async fn rotate(&self, _old_id: Uuid, _new_hash: &str) -> Result<(), AppError> {
        Ok(())
    }

    async fn revoke_all_for_user(&self, _user_id: Uuid) -> Result<(), AppError> {
        Ok(())
    }
}

#[derive(Clone, Default)]
struct MockIdentityRepo {
    state: Arc<Mutex<MockIdentityState>>,
}

#[derive(Default)]
struct MockIdentityState {
    by_provider_subject: HashMap<(String, String), UserIdentityRow>,
    created: Vec<(Uuid, String, String)>,
}

#[async_trait]
impl AuthUserIdentityRepo for MockIdentityRepo {
    async fn find_by_provider_subject(
        &self,
        provider: &str,
        provider_subject: &str,
    ) -> Result<Option<UserIdentityRow>, AppError> {
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

#[derive(Clone)]
struct MockPasswordHasher;

impl PasswordHasher for MockPasswordHasher {
    fn hash(&self, _password: &str) -> Result<String, AppError> {
        Ok("hash".to_string())
    }

    fn verify(&self, _password: &str, _password_hash: &str) -> Result<bool, AppError> {
        Ok(true)
    }
}

#[derive(Clone)]
struct MockTokenIssuer;

impl TokenIssuer for MockTokenIssuer {
    fn generate(&self, _user_id: Uuid) -> Result<String, AppError> {
        Ok("token".to_string())
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
    refresh_repo: MockRefreshTokenRepo,
    subscription_repo: MockSubscriptionRepo,
    identity_repo: MockIdentityRepo,
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
        refresh_repo,
        subscription_repo,
        identity_repo,
        MockPasswordHasher,
        MockTokenIssuer,
        verifier,
    )
}

#[tokio::test]
async fn oauth_login_creates_user_and_identity() {
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
    let verifier = MockOauthVerifier {
        claims: OAuthClaims {
            provider: "google".to_string(),
            subject: "subject-1".to_string(),
            email: Some("alice@example.com".to_string()),
            email_verified: Some(true),
            name: Some("Alice".to_string()),
            picture_url: None,
        },
    };

    let service = make_service(
        user_repo.clone(),
        settings_repo.clone(),
        pocket_repo.clone(),
        refresh_repo.clone(),
        subscription_repo.clone(),
        identity_repo.clone(),
        verifier,
    );

    let resp = service
        .oauth_login(OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "token".to_string(),
            username: None,
            base_currency: Some("USD".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(resp.message, "Login successful");
    assert!(user_repo.state.lock().unwrap().created_user.is_some());
    assert_eq!(settings_repo.set_calls.lock().unwrap().len(), 1);
    assert_eq!(pocket_repo.calls.lock().unwrap().len(), 1);
    assert_eq!(subscription_repo.calls.lock().unwrap().len(), 1);
    assert_eq!(identity_repo.state.lock().unwrap().created.len(), 1);
    assert_eq!(refresh_repo.state.lock().unwrap().created.len(), 1);
}

#[tokio::test]
async fn oauth_login_links_existing_identity() {
    unsafe { std::env::set_var("GOOGLE_CLIENT_ID", "client-id") };

    let identity_repo = MockIdentityRepo::default();
    let existing_identity = UserIdentityRow {
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
            existing_identity,
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
        MockOauthVerifier {
            claims: OAuthClaims {
                provider: "google".to_string(),
                subject: "subject-1".to_string(),
                email: Some("alice@example.com".to_string()),
                email_verified: Some(true),
                name: None,
                picture_url: None,
            },
        },
    );

    let resp = service
        .oauth_login(OAuthLoginRequest {
            provider: "google".to_string(),
            id_token: "token".to_string(),
            username: None,
            base_currency: None,
        })
        .await
        .unwrap();

    assert_eq!(resp.message, "Login successful");
}
