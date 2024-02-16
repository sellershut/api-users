use api_core::{api::CoreError, reexports::uuid::Uuid, Account, Session, User, VerificationToken};
use serde::{Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Id};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntityUser {
    pub id: RecordId,
    pub name: Option<String>,
    pub email: String,
    pub email_verified: Option<OffsetDateTime>,
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntitySession {
    pub id: RecordId,
    pub user: RecordId,
    pub expires: OffsetDateTime,
    pub session_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntityAccount {
    pub id: RecordId,
    pub user: RecordId,
    #[serde(rename = "type")]
    pub account_type: String,
    pub provider: String,
    pub provider_account_id: String,
    pub refresh_token: Option<String>,
    pub access_token: Option<String>,
    pub expires_at: Option<usize>,
    pub id_token: String,
    pub scope: String,
    pub session_state: String,
    pub token_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntityVerificationToken {
    pub id: RecordId,
    pub identifier: String,
    pub token: String,
    pub expires: OffsetDateTime,
}

impl TryFrom<DatabaseEntityVerificationToken> for VerificationToken {
    type Error = CoreError;

    fn try_from(value: DatabaseEntityVerificationToken) -> Result<Self, Self::Error> {
        let id = record_id_to_uuid(&value.id)?;
        Ok(VerificationToken {
            id,
            identifier: value.identifier,
            token: value.token,
            expires: value.expires,
        })
    }
}

impl TryFrom<DatabaseEntityAccount> for Account {
    type Error = CoreError;

    fn try_from(value: DatabaseEntityAccount) -> Result<Self, Self::Error> {
        let id = record_id_to_uuid(&value.id)?;
        let user = record_id_to_uuid(&value.user)?;

        Ok(Account {
            id,
            user,
            account_type: value.account_type,
            provider: value.provider,
            provider_account_id: value.provider_account_id,
            refresh_token: value.refresh_token,
            access_token: value.access_token,
            expires_at: value.expires_at,
            id_token: value.id_token,
            scope: value.scope,
            session_state: value.session_state,
            token_type: value.token_type,
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
            expires: value.expires,
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
            name: entity.name,
            email: entity.email,
            email_verified: entity.email_verified,
            image: entity.image,
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
