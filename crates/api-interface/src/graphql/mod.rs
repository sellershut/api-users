pub(crate) mod mutation;
pub(crate) mod query;
pub(crate) mod subscription;

use api_database::Client;
use async_graphql::Context;
use tracing::error;

pub(crate) fn extract_db<'a>(context: &'a Context) -> async_graphql::Result<&'a Client> {
    context.data::<Client>().map_err(|db| {
        error!("{}", db.message);
        "Internal database error".into()
    })
}
