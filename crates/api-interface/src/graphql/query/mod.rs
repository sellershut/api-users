use async_graphql::connection::{Connection, EmptyFields};

pub(crate) mod pagination;
pub(crate) mod user;

#[derive(async_graphql::MergedObject, Default)]
pub struct Query(user::UserQuery);

pub(crate) type ConnectionResult<T> = async_graphql::Result<
    Connection<pagination::Base64Cursor, T, pagination::ConnectionFields, EmptyFields>,
>;

/// Relay-compliant connection parameters to page results by cursor/page size
pub struct Params {
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
}

impl Params {
    pub fn new(
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> async_graphql::Result<Self> {
        if last.is_none() && first.is_none() {
            return Err("One of 'first' or 'last' should be provided".into());
        }

        if after.is_some() && before.is_some() {
            return Err("Only one or none of 'after' or 'before' should be provided".into());
        }

        Ok(Self {
            after,
            before,
            first,
            last,
        })
    }
}
