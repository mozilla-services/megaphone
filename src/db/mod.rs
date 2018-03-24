pub mod schema;
pub mod models;

use std::ops::Deref;
use std::result::Result as StdResult;

use diesel::Connection;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Error, Pool, PooledConnection};
use failure::err_msg;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Config, Outcome, Request, State};

use error::Result;

pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;

embed_migrations!();

/// Run the diesel embedded migrations
///
/// Mysql DDL statements implicitly commit which could disrupt MysqlPool's
/// begin_test_transaction during tests. So this runs on its own separate conn.
pub fn run_embedded_migrations(config: &Config) -> Result<()> {
    let database_url = config
        .get_str("database_url")
        .map_err(|_| err_msg("Invalid or undefined ROCKET_DATABASE_URL"))?
        .to_string();
    let conn = MysqlConnection::establish(&database_url)?;
    Ok(embedded_migrations::run(&conn)?)
}

pub fn pool_from_config(config: &Config) -> Result<MysqlPool> {
    let database_url = config
        .get_str("database_url")
        .map_err(|_| err_msg("Invalid or undefined ROCKET_DATABASE_URL"))?
        .to_string();
    let max_size = config.get_int("database_pool_max_size").unwrap_or(10) as u32;
    let use_test_transactions = config
        .get_bool("database_use_test_transactions")
        .unwrap_or(false);

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let mut builder = Pool::builder().max_size(max_size);
    if use_test_transactions {
        builder = builder.connection_customizer(Box::new(TestTransactionCustomizer));
    }
    Ok(builder.build(manager)?)
}

pub struct Conn(pub PooledConnection<ConnectionManager<MysqlConnection>>);

impl Deref for Conn {
    type Target = MysqlConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Conn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, ()> {
        let pool = request.guard::<State<MysqlPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Conn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

#[derive(Debug)]
struct TestTransactionCustomizer;

impl CustomizeConnection<MysqlConnection, Error> for TestTransactionCustomizer {
    fn on_acquire(&self, conn: &mut MysqlConnection) -> StdResult<(), Error> {
        conn.begin_test_transaction().map_err(Error::QueryError)
    }
}
