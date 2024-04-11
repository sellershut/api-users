use api_core::{api::CoreError, reexports::uuid::Uuid, AccountProvider, Session, User, UserType};
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
    #[serde(rename = "type")]
    pub user_type: UserType,
    pub phone_number: Option<String>,
    pub created: isize,
    pub updated: isize,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntitySession {
    pub id: RecordId,
    #[serde(rename = "in")]
    pub in_field: RecordId,
    pub out: DatabaseEntityAccountProvider,
    pub expires_at: OffsetDateTime,
    pub session_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct DatabaseEntityAccountProvider {
    pub id: RecordId,
    pub name: String,
}

impl TryFrom<DatabaseEntityAccountProvider> for AccountProvider {
    type Error = CoreError;

    fn try_from(value: DatabaseEntityAccountProvider) -> Result<Self, Self::Error> {
        let id = record_id_to_uuid(&value.id)?;

        Ok(AccountProvider {
            id,
            name: value.name,
        })
    }
}

impl TryFrom<DatabaseEntitySession> for Session {
    type Error = CoreError;

    fn try_from(value: DatabaseEntitySession) -> Result<Self, Self::Error> {
        let user_id = record_id_to_uuid(&value.in_field)?;
        let account_provider_id = record_id_to_uuid(&value.out.id)?;

        Ok(Session {
            expires_at: value.expires_at,
            session_token: value.session_token,
            account_provider: AccountProvider {
                id: account_provider_id,
                name: value.out.name,
            },
            user_id,
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
            user_type: entity.user_type,
            phone_number: entity.phone_number,
            created: entity.created,
            updated: entity.updated,
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
