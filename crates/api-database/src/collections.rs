use serde::{Deserialize, Serialize};
use surrealdb::{
    opt::{IntoResource, Resource},
    sql::Table,
};

#[non_exhaustive]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Collection {
    User,
    #[serde(rename = "account_provider")]
    AccountProvider,
    #[serde(rename = "user_account")]
    UserAccount,
    #[serde(rename = "user_session")]
    UserSession,
}

impl std::fmt::Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Collection::User => "user",
                Collection::AccountProvider => "account_provider",
                Collection::UserAccount => "user_account",
                Collection::UserSession => "user_session",
            }
        )
    }
}

impl<R> IntoResource<Vec<R>> for Collection {
    fn into_resource(self) -> Result<Resource, surrealdb::Error> {
        Ok(Resource::Table(Table(self.to_string())))
    }
}
