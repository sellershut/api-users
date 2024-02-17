use api_core::{
    api::{CoreError, QueryVerificationToken},
    VerificationToken,
};
use std::fmt::Debug;

use crate::{collections::Collections, map_db_error, Client};

impl QueryVerificationToken for Client {
    async fn get_verification_token(
        &self,
        token: impl AsRef<str> + Send + Debug,
        identifier: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<VerificationToken>, CoreError> {
        let token = token.as_ref();
        let identifier = identifier.as_ref();

        let mut resp = self
            .client
            .query(format!(
                "SELECT * FROM {} WHERE token = '{token}' AND identifier = '{identifier}'",
                Collections::VerificationToken
            ))
            .await
            .map_err(map_db_error)?;

        let token: Option<VerificationToken> = resp.take(0).map_err(map_db_error)?;
        Ok(token)
    }
}
