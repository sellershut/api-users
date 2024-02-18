use api_core::{api::QueryUsers, User};
use async_graphql::{Context, Object, Subscription};
use futures_util::{Stream, StreamExt};

use crate::graphql::{extract_db, mutation::MutationType, subscription::UserChanged};

use super::broker::SimpleBroker;

#[derive(Default)]
pub struct UserSubscription;

#[Subscription]
impl UserSubscription {
    async fn users(&self, mutation_type: Option<MutationType>) -> impl Stream<Item = UserChanged> {
        SimpleBroker::<UserChanged>::subscribe().filter(move |event| {
            let res = if let Some(mutation_type) = mutation_type {
                event.mutation_type == mutation_type
            } else {
                true
            };
            async move { res }
        })
    }
}

#[Object]
impl UserChanged {
    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn user(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<User>> {
        let database = extract_db(ctx)?;
        let user = database.get_user_by_id(&self.id).await?;

        Ok(user)
    }
}
