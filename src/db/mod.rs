pub mod schema;
pub mod models;

use std::ops::Deref;

use diesel::mysql::MysqlConnection;
use r2d2;
use r2d2_diesel::ConnectionManager;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};

pub type Pool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

pub fn init_pool(database_url: String, max_size: u32) -> Pool {
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("db pool")  // XXX:
}

pub struct Conn(pub r2d2::PooledConnection<ConnectionManager<MysqlConnection>>);

impl Deref for Conn {
    type Target = MysqlConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Conn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Conn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Conn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}
