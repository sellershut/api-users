use api_core::{
    api::{MutateUsers, Uuid},
    User,
};
use api_database::Client;
use async_graphql::{Context, Object};
use tracing::instrument;

use crate::graphql::subscription::{broker::SimpleBroker, UserChanged};

#[derive(Default, Debug)]
pub struct UserMutation;

#[Object]
impl UserMutation {
    #[instrument(skip(ctx), err(Debug))]
    async fn create_user(&self, ctx: &Context<'_>, input: User) -> async_graphql::Result<User> {
        let database = ctx.data::<Client>()?;

        match database.create_user(&input).await {
            Ok(user) => {
                SimpleBroker::publish(UserChanged {
                    mutation_type: super::MutationType::Created,
                    id: user.id,
                });

                Ok(user)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn update_user(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: User,
    ) -> async_graphql::Result<Option<User>> {
        let database = ctx.data::<Client>()?;

        match database.update_user(&id, &input).await {
            Ok(user) => {
                SimpleBroker::publish(UserChanged {
                    mutation_type: super::MutationType::Updated,
                    id,
                });
                Ok(user)
            }
            Err(e) => Err(e.into()),
        }
    }

    #[instrument(skip(ctx), err(Debug))]
    async fn delete_user(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
    ) -> async_graphql::Result<Option<User>> {
        let database = ctx.data::<Client>()?;

        match database.delete_user(&id).await {
            Ok(user) => {
                SimpleBroker::publish(UserChanged {
                    mutation_type: super::MutationType::Deleted,
                    id,
                });
                Ok(user)
            }
            Err(e) => Err(e.into()),
        }
    }
}
