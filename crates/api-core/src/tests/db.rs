use std::fmt::Debug;

use uuid::Uuid;

use crate::{
    api::{CoreError, LocalMutateUsers, LocalQueryUsers, MutateUsers, QueryUsers},
    Session, User,
};

pub struct SampleDb;
pub struct SampleDbSend;

impl LocalQueryUsers for SampleDb {
    async fn get_users(&self) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_user_by_id(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn get_user_by_email(
        &self,
        _id: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn search(
        &self,
        _query: impl AsRef<str> + Debug + Send,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_user_by_account(
        &self,
        _provider: impl AsRef<str> + Send + Debug,
        _provider_account_id: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn get_session_and_user(
        &self,
        _session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<(User, Session)>, CoreError> {
        Ok(None)
    }
}

impl LocalMutateUsers for SampleDb {
    async fn create_user(&self, user: &User) -> Result<User, CoreError> {
        Ok(user.clone())
    }

    async fn update_user(&self, _id: &Uuid, _data: &User) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn delete_user(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }
}

impl MutateUsers for SampleDbSend {
    async fn create_user(&self, user: &User) -> Result<User, CoreError> {
        Ok(user.to_owned())
    }

    async fn update_user(&self, _id: &Uuid, data: &User) -> Result<Option<User>, CoreError> {
        Ok(Some(data.to_owned()))
    }

    async fn delete_user(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }
}

impl QueryUsers for SampleDbSend {
    async fn get_users(&self) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_user_by_id(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn get_user_by_email(
        &self,
        _email: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn get_user_by_account(
        &self,
        _provider: impl AsRef<str> + Send + Debug,
        _provider_account_id: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn search(
        &self,
        _query: impl AsRef<str> + Send + Debug,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_session_and_user(
        &self,
        _session_token: impl AsRef<str> + Send + Debug,
    ) -> Result<Option<(User, Session)>, CoreError> {
        Ok(None)
    }
}
