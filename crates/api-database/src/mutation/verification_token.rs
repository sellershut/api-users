use api_core::{
    api::{CoreError, MutateVerificationToken},
    VerificationToken,
};
use std::fmt::Debug;
use uuid::Uuid;

use crate::{
    collections::Collections, entity::DatabaseEntityVerificationToken, map_db_error, Client,
};

impl MutateVerificationToken for Client {
    async fn create_verification_token(
        &self,
        vft: &VerificationToken,
    ) -> Result<VerificationToken, CoreError> {
        let id = Uuid::now_v7().to_string();
        let item: Option<DatabaseEntityVerificationToken> = self
            .client
            .create((Collections::VerificationToken.to_string(), id))
            .content(vft)
            .await
            .map_err(map_db_error)?;

        let token = item.expect("token to have been created");
        VerificationToken::try_from(token)
    }

    async fn use_verification_token(
        &self,
        identifier: impl AsRef<str> + Send + Debug,
        token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<VerificationToken>, CoreError> {
        let identifier = identifier.as_ref();
        let token = token.as_ref();

        let mut res = self
            .client
            .query(format!(
                "SELECT * FROM {} WHERE identifier = {identifier} AND token = {token}",
                Collections::VerificationToken
            ))
            .await
            .map_err(map_db_error)?;

        let vft: Option<DatabaseEntityVerificationToken> = res.take(0).map_err(map_db_error)?;

        match vft {
            Some(vft) => Ok(Some(VerificationToken::try_from(vft)?)),
            None => Err(CoreError::Other("no token was found in db".to_owned())),
        }
    }
}
