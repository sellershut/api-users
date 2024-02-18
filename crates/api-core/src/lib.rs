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
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub user_type: UserType,
    pub phone_number: Option<String>,
    #[cfg_attr(
        feature = "async-graphql",
        graphql(default_with = "default_date_time()")
    )]
    pub created_at: OffsetDateTime,
    pub updated_at: Option<OffsetDateTime>,
}

#[cfg(feature = "async-graphql")]
fn default_date_time() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(Enum))]
pub enum UserType {
    Individual,
    Company,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(InputObject, SimpleObject))]
#[cfg_attr(feature = "async-graphql", graphql(input_name = "SessionInput"))]
pub struct Session {
    #[cfg_attr(feature = "async-graphql", graphql(skip_input))]
    pub id: Uuid,
    pub user: Uuid,
    pub expires_at: OffsetDateTime,
    pub session_token: String,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "async-graphql", derive(InputObject, SimpleObject))]
#[cfg_attr(feature = "async-graphql", graphql(input_name = "AccountInput"))]
pub struct Account {
    #[cfg_attr(feature = "async-graphql", graphql(skip_input))]
    pub id: Uuid,
    pub user: Uuid,
    pub provider: String,
    pub provider_account_id: String,
}

pub mod reexports {
    pub use uuid;
}

#[cfg(test)]
mod tests;
