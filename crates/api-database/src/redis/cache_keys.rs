use std::fmt::Display;

use api_core::reexports::uuid::Uuid;
use redis::ToRedisArgs;

#[derive(Clone, Copy, Debug)]
pub enum CacheKey<'a> {
    AllUsers,
    Session {
        token: &'a str,
    },
    UserById {
        id: &'a Uuid,
    },
    UserByEmail {
        email: &'a str,
    },
    UserByAccount {
        provider: &'a str,
        provider_account_id: &'a str,
    },
}

impl Display for CacheKey<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "users|session:{}",
            match self {
                CacheKey::AllUsers => "users|all".to_string(),
                CacheKey::UserById { id } => format!("user|id={id}"),
                CacheKey::UserByEmail { email } => format!("user|email={email}"),
                CacheKey::UserByAccount {
                    provider,
                    provider_account_id,
                } => {
                    format!("account|provider={provider}|provider_account_id={provider_account_id}")
                }
                CacheKey::Session { token } => {
                    format!("session={token}")
                }
            }
        )
    }
}

impl ToRedisArgs for CacheKey<'_> {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(self.to_string().as_bytes())
    }
}
