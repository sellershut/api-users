use std::fmt;

use api_core::{api::CoreError, reexports::uuid::Uuid, AccountProvider, Session, User, UserType};
use serde::{de, Deserialize, Serialize};
use surrealdb::{opt::RecordId, sql::Id};
use time::{format_description::well_known::Iso8601, OffsetDateTime};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntityUser {
    pub id: RecordId,
    pub username: String,
    pub email: String,
    pub name: Option<String>,
    pub avatar: Option<String>,
    #[serde(rename = "type")]
    pub user_type: UserType,
    pub phone_number: Option<String>,
    #[serde(deserialize_with = "deserialize_date_time")]
    pub created: OffsetDateTime,
    #[serde(deserialize_with = "deserialize_date_time")]
    pub updated: OffsetDateTime,
}

fn deserialize_date_time<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct OffsetDateTimeVisitor;

    impl<'de> de::Visitor<'de> for OffsetDateTimeVisitor {
        type Value = OffsetDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string containing a ISO8601 date")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            OffsetDateTime::parse(v, &Iso8601::DEFAULT).map_err(E::custom)
        }
    }

    deserializer.deserialize_any(OffsetDateTimeVisitor)
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct DatabaseEntitySession {
    pub id: RecordId,
    #[serde(rename = "in")]
    pub in_field: RecordId,
    pub out: DatabaseEntityAccountProvider,
    #[serde(deserialize_with = "deserialize_date_time")]
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
