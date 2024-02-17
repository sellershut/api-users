pub mod api;

#[cfg(feature = "async-graphql")]
use async_graphql::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use time::OffsetDateTime;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(InputObject, SimpleObject))]
#[cfg_attr(feature = "async-graphql", graphql(input_name = "UserInput"))]
pub struct User {
    #[cfg_attr(feature = "async-graphql", graphql(skip_input))]
    pub id: Uuid,
    pub name: Option<String>,
    pub email: String,
    pub email_verified: Option<OffsetDateTime>,
    pub image: Option<String>,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(InputObject, SimpleObject))]
#[cfg_attr(feature = "async-graphql", graphql(input_name = "SessionInput"))]
pub struct Session {
    pub id: Uuid,
    pub user: Uuid,
    pub expires: OffsetDateTime,
    pub session_token: String,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(InputObject, SimpleObject))]
#[cfg_attr(
    feature = "async-graphql",
    graphql(input_name = "VerificationTokenInput")
)]
pub struct VerificationToken {
    #[cfg_attr(feature = "async-graphql", graphql(skip))]
    pub id: Uuid,
    pub identifier: String,
    pub token: String,
    pub expires: OffsetDateTime,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(InputObject, SimpleObject))]
#[cfg_attr(feature = "async-graphql", graphql(input_name = "AccountInput"))]
pub struct Account {
    #[cfg_attr(feature = "async-graphql", graphql(skip_input))]
    pub id: Uuid,
    pub user: Uuid,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub account_type: String,
    pub provider: String,
    pub provider_account_id: String,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub expires_at: Option<usize>,
    pub id_token: String,
    pub scope: String,
    pub session_state: String,
    pub token_type: String,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AccountType {
    Oauth,
    Oidc,
    Email,
    Webauthn,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AccountType::Oauth => "oauth",
                AccountType::Oidc => "oidc",
                AccountType::Email => "email",
                AccountType::Webauthn => "webauthn",
            }
        )
    }
}

pub mod reexports {
    pub use uuid;
}

#[cfg(test)]
mod tests;
