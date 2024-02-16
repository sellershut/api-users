use std::{fmt::Debug, str::FromStr};

use uuid::Uuid;

use crate::{
    api::{CoreError, LocalMutateUsers, LocalQueryUsers, MutateCategories, QueryCategories},
    User,
};

pub struct SampleDb;
pub struct SampleDbSend;

impl LocalQueryUsers for SampleDb {
    async fn get_categories(&self) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_sub_categories(
        &self,
        _id: Option<&Uuid>,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_category_by_id(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn search(
        &self,
        _query: impl AsRef<str> + Debug + Send,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }
}

impl LocalMutateUsers for SampleDb {
    async fn create_category(&self, category: &User) -> Result<User, CoreError> {
        Ok(category.to_owned())
    }

    async fn update_category(&self, id: &Uuid, data: &User) -> Result<Option<User>, CoreError> {
        if id.as_ref().is_empty() {
            Err(CoreError::from_str("Id cannot be empty")?)
        } else {
            Ok(Some(data.to_owned()))
        }
    }

    async fn delete_category(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }
}

impl MutateCategories for SampleDbSend {
    async fn create_category(&self, category: &User) -> Result<User, CoreError> {
        Ok(category.to_owned())
    }

    async fn update_category(&self, _id: &Uuid, data: &User) -> Result<Option<User>, CoreError> {
        Ok(Some(data.to_owned()))
    }

    async fn delete_category(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }
}

impl QueryCategories for SampleDbSend {
    async fn get_categories(&self) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_sub_categories(
        &self,
        _id: Option<&Uuid>,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }

    async fn get_category_by_id(&self, _id: &Uuid) -> Result<Option<User>, CoreError> {
        Ok(None)
    }

    async fn search(
        &self,
        _query: impl AsRef<str> + Debug + Send,
    ) -> Result<impl ExactSizeIterator<Item = User>, CoreError> {
        Ok([].into_iter())
    }
}
