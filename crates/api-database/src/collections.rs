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
    Session,
    Account,
}

impl std::fmt::Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Collection::User => "user",
                Collection::Session => "session",
                Collection::Account => "oauth_account",
            }
        )
    }
}

impl<R> IntoResource<Vec<R>> for Collection {
    fn into_resource(self) -> Result<Resource, surrealdb::Error> {
        Ok(Resource::Table(Table(self.to_string())))
    }
}
