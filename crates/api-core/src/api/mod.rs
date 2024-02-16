mod error;
pub use std::fmt::Debug;

use crate::{Account, Session, User, VerificationToken};

pub use error::*;
pub use uuid::Uuid;

#[trait_variant::make(QueryUsers: Send)]
pub trait LocalQueryUsers {
    async fn get_users(&self) -> Result<impl ExactSizeIterator<Item = User>, CoreError>;
    async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>, CoreError>;
    async fn get_user_by_email(
        &self,
        email: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError>;
    async fn get_user_by_account(
        &self,
        provider: impl AsRef<str> + Send + Debug,
        provider_account_id: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError>;
    async fn search(
        &self,
        query: impl AsRef<str> + Send + Debug,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError>;
    async fn get_session_and_user(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<(User, Session)>, CoreError>;
}

#[trait_variant::make(MutateUsers: Send)]
pub trait LocalMutateUsers {
    async fn create_user(&self, user: &User) -> Result<User, CoreError>;
    async fn update_user(&self, id: &Uuid, data: &User) -> Result<Option<User>, CoreError>;
    async fn delete_user(&self, id: &Uuid) -> Result<Option<User>, CoreError>;
}

#[trait_variant::make(MutateAccounts: Send)]
pub trait LocalMutateAccounts {
    async fn link_account(&self, account: &Account) -> Result<Account, CoreError>;
    async fn unlink_account(
        &self,
        provider: impl AsRef<str> + Send + Debug,
        provider_account_id: impl AsRef<str> + Send + Debug,
    ) -> Result<(), CoreError>;
}

#[trait_variant::make(MutateSessions: Send)]
pub trait LocalMutateSessions {
    async fn create_session(&self, session: &Session) -> Result<Session, CoreError>;
    async fn update_session(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
        session: &Session,
    ) -> Result<Option<Session>, CoreError>;
    async fn delete_session(
        &self,
        session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<(), CoreError>;
}

#[trait_variant::make(MutateVerificationToken: Send)]
pub trait LocalMutateVerificationToken {
    async fn create_verification_token(
        &self,
        session: &VerificationToken,
    ) -> Result<Account, CoreError>;
    async fn use_verification_token(
        &self,
        identifier: impl AsRef<str> + Send + Debug,
        token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<VerificationToken>, CoreError>;
}
