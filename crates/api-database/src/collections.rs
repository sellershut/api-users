use surrealdb::{
    opt::{IntoResource, Resource},
    sql::Table,
};

#[non_exhaustive]
pub(crate) enum Collections {
    User,
    Session,
    Account,
}

impl std::fmt::Display for Collections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Collections::User => "user",
                Collections::Session => "session",
                Collections::Account => "oauth_account",
            }
        )
    }
}

impl<R> IntoResource<Vec<R>> for Collections {
    fn into_resource(self) -> Result<Resource, surrealdb::Error> {
        Ok(Resource::Table(Table(self.to_string())))
    }
}
