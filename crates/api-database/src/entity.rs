use api_core::{api::CoreError, reexports::uuid::Uuid, Account, Session, User};
use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Id};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntityUser {
    pub id: RecordId,
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntitySession {
    pub id: RecordId,
    pub user: RecordId,
    pub expires_at: OffsetDateTime,
    pub session_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct DatabaseEntityAccount {
    pub id: RecordId,
    pub user: RecordId,
    pub provider: String,
    pub provider_account_id: String,
}

impl TryFrom<DatabaseEntityAccount> for Account {
    type Error = CoreError;

    fn try_from(value: DatabaseEntityAccount) -> Result<Self, Self::Error> {
        let id = record_id_to_uuid(&value.id)?;
        let user = record_id_to_uuid(&value.user)?;

        Ok(Account {
            id,
            user,
            provider: value.provider,
            provider_account_id: value.provider_account_id,
        })
    }
}

impl TryFrom<DatabaseEntitySession> for Session {
    type Error = CoreError;

    fn try_from(value: DatabaseEntitySession) -> Result<Self, Self::Error> {
        let id = record_id_to_uuid(&value.id)?;
        let user = record_id_to_uuid(&value.user)?;

        Ok(Session {
            id,
            user,
            expires_at: value.expires_at,
            session_token: value.session_token,
        })
    }
}

impl TryFrom<DatabaseEntityUser> for User {
    type Error = CoreError;

    fn try_from(entity: DatabaseEntityUser) -> Result<Self, Self::Error> {
        let id = record_id_to_uuid(&entity.id)?;

        Ok(User {
            id,
            username: entity.username,
            email: entity.email,
            name: entity.name,
            avatar: entity.avatar,
        })
    }
}

pub(crate) fn record_id_to_uuid(id: &RecordId) -> Result<Uuid, CoreError> {
    let id_to_string = |id: &Id| -> String {
        let id = id.to_raw();
        id.split(':')
            .next()
            .unwrap_or(&id)
            .chars()
            .filter(|&c| c != '⟨' && c != '⟩')
            .collect()
    };

    let pk = id_to_string(&id.id);
    Ok(Uuid::parse_str(&pk)?)
}
