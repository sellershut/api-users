pub(crate) mod broker;
pub(crate) mod user;
use api_core::reexports::uuid::Uuid;

use super::mutation::MutationType;

#[derive(async_graphql::MergedSubscription, Default)]
pub struct Subscription(user::UserSubscription);

#[derive(Debug, Clone)]
pub(crate) struct UserChanged {
    pub mutation_type: MutationType,
    pub id: Uuid,
}
