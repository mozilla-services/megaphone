/// Authentication/Authorization
///
/// Driven from Bearer tokens defined in the rocket Config, keyed by a
/// user id (a broadcaster or reader id).
///
/// Broadcasts are id'd by 'broadcaster_id/bchannel_id'. Broadcasters can only
/// create new broadcasts under their own broadcaster_id. Readers can read all
/// broadcasts.
use std::collections::HashMap;

use rocket::{Config, Request, State};
use rocket::config::Value;

use db::models::{Broadcaster, Reader};
use error::{HandlerErrorKind, HandlerResult, Result};

/// Tokens mapped to an authorized id, from rocket's Config
type AuthToken = String;
type UserId = String;

/// Grouping/role of authorization
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Group {
    Broadcaster,
    Reader,
}

impl Group {
    /// Entry name in rocket Config where tokens are loaded from
    fn config_name(&self) -> &'static str {
        match *self {
            Group::Broadcaster => "broadcaster_auth",
            Group::Reader => "reader_auth",
        }
    }
}

#[derive(Debug)]
pub struct BearerTokenAuthenticator {
    users: HashMap<AuthToken, UserId>,
    groups: HashMap<UserId, Group>,
}

impl BearerTokenAuthenticator {
    pub fn from_config(config: &Config) -> Result<BearerTokenAuthenticator> {
        let mut authenticator = BearerTokenAuthenticator {
            users: HashMap::new(),
            groups: HashMap::new(),
        };
        authenticator.load_auth_from_config(Group::Broadcaster, config)?;
        authenticator.load_auth_from_config(Group::Reader, config)?;
        Ok(authenticator)
    }

    /// Load the Group's auth configuration
    fn load_auth_from_config(&mut self, group: Group, config: &Config) -> Result<()> {
        let name = group.config_name();
        let auth_config = config
            .get_table(name)
            .map_err(|_| format_err!("Invalid or undefined ROCKET_{}", name.to_uppercase()))?;

        for (user_id, tokens_val) in auth_config {
            if let Some(dupe) = self.groups.get(user_id) {
                Err(format_err!(
                    "Invalid {} user: {:?} dupe user in: {}",
                    name,
                    user_id,
                    dupe.config_name()
                ))?
            }
            self.groups.insert(user_id.to_string(), group);

            let tokens = tokens_val.as_array().ok_or(format_err!(
                "Invalid {} token array for: {:?}",
                name,
                user_id
            ))?;
            self.load_tokens(user_id, group, tokens)?;
        }
        Ok(())
    }

    fn load_tokens(&mut self, user_id: &UserId, group: Group, tokens: &[Value]) -> Result<()> {
        let name = group.config_name();
        for element in tokens {
            let token =
                element
                    .as_str()
                    .ok_or(format_err!("Invalid {} token for: {:?}", name, user_id))?;
            if let Some(dupe) = self.users.get(token) {
                Err(format_err!(
                    "Invalid {} token for: {:?} dupe in: {:?} ({:?})",
                    name,
                    user_id,
                    dupe,
                    token
                ))?
            }
            self.users.insert(token.to_string(), user_id.to_string());
        }
        Ok(())
    }

    /// Determine if Bearer token header is for an authenticated user
    fn authenticated_user(&self, credentials: &str) -> HandlerResult<(UserId, Group)> {
        let parts: Vec<_> = credentials.splitn(2, ' ').collect();
        if parts.len() != 2 || parts[0].to_lowercase() != "bearer" {
            Err(HandlerErrorKind::InvalidAuth)?
        }

        let user_id = self.users
            .get(parts[1])
            .ok_or_else(|| HandlerErrorKind::InvalidAuth)?;
        // Authenticated
        let group = self.groups
            .get(user_id)
            .ok_or_else(|| HandlerErrorKind::InternalError)?;
        Ok((user_id.to_string(), *group))
    }
}

fn authenticated_user(request: &Request) -> HandlerResult<(UserId, Group)> {
    let credentials = request
        .headers()
        .get_one("Authorization")
        .ok_or_else(|| HandlerErrorKind::MissingAuth)?;
    request
        .guard::<State<BearerTokenAuthenticator>>()
        .success_or(HandlerErrorKind::InternalError)?
        .authenticated_user(credentials)
}

pub fn authorized_broadcaster(request: &Request) -> HandlerResult<Broadcaster> {
    let (id, group) = authenticated_user(request)?;

    // param should be guaranteed on the path when we're called
    let for_broadcast_id = request
        .get_param::<String>(0)
        .map_err(HandlerErrorKind::RocketError)?;

    if group == Group::Broadcaster && id == for_broadcast_id {
        // Authorized
        Ok(Broadcaster::new(id))
    } else {
        Err(HandlerErrorKind::Unauthorized)?
    }
}

pub fn authorized_reader(request: &Request) -> HandlerResult<Reader> {
    let (id, group) = authenticated_user(request)?;
    if group == Group::Reader {
        // Authorized
        Ok(Reader::new(id))
    } else {
        Err(HandlerErrorKind::Unauthorized)?
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use rocket::config::{Config, Environment};

    use super::{BearerTokenAuthenticator, Group};

    #[test]
    fn test_basic() {
        let mut bauth = HashMap::new();
        bauth.insert("foo", vec!["bar"]);
        bauth.insert("baz", vec!["quux"]);
        let mut rauth = HashMap::new();
        rauth.insert("otto", vec!["push"]);

        let config = Config::build(Environment::Development)
            .extra("broadcaster_auth", bauth)
            .extra("reader_auth", rauth)
            .unwrap();
        let authenicator = BearerTokenAuthenticator::from_config(&config).unwrap();

        assert_eq!(
            authenicator.authenticated_user("Bearer quux").unwrap(),
            ("baz".to_string(), Group::Broadcaster)
        );
        assert_eq!(
            authenicator.authenticated_user("Bearer push").unwrap(),
            ("otto".to_string(), Group::Reader)
        );
        assert!(authenicator.authenticated_user("Bearer mega").is_err());
    }

    #[test]
    fn test_dupe_token() {
        let mut bauth = HashMap::new();
        bauth.insert("foo", vec!["bar"]);
        bauth.insert("baz", vec!["bar"]);
        let config = Config::build(Environment::Development)
            .extra("broadcaster_auth", bauth)
            .extra("reader_auth", HashMap::<&str, Vec<&str>>::new())
            .unwrap();
        assert!(BearerTokenAuthenticator::from_config(&config).is_err());
    }

    #[test]
    fn test_dupe_token2() {
        let mut bauth = HashMap::new();
        bauth.insert("foo", vec!["bar"]);
        let mut rauth = HashMap::new();
        rauth.insert("baz", vec!["quux", "bar"]);
        let config = Config::build(Environment::Development)
            .extra("broadcaster_auth", bauth)
            .extra("reader_auth", rauth)
            .unwrap();
        assert!(BearerTokenAuthenticator::from_config(&config).is_err());
    }

    #[test]
    fn test_dupe_user() {
        let mut bauth = HashMap::new();
        bauth.insert("foo", vec!["bar"]);
        let mut rauth = HashMap::new();
        rauth.insert("foo", vec!["baz"]);
        let config = Config::build(Environment::Development)
            .extra("broadcaster_auth", bauth)
            .extra("reader_auth", rauth)
            .unwrap();
        assert!(BearerTokenAuthenticator::from_config(&config).is_err());
    }
}
