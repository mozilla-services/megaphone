use failure::ResultExt;
use diesel::{replace_into, RunQueryDsl};
use diesel::mysql::MysqlConnection;

use super::schema::versionv1;
use error::{HandlerErrorKind, HandlerResult};

#[derive(Debug, Queryable, Insertable)]
#[table_name = "versionv1"]
pub struct Version {
    pub service_id: String, // combination of the broadcast + '/' + collection ids
    pub version: String,    // version information
}

/// An authorized broadcaster
pub struct Broadcaster {
    pub id: String,
}

impl Broadcaster {
    pub fn broadcast_new_version(
        &self,
        conn: &MysqlConnection,
        collection_id: String,
        version: String,
    ) -> HandlerResult<usize> {
        let new_version = Version {
            service_id: format!("{}/{}", self.id, collection_id),
            version: version,
        };
        Ok(replace_into(versionv1::table)
            .values(&new_version)
            .execute(conn)
            .context(HandlerErrorKind::DBError)?)
    }
}

// An authorized reader of current broadcasts
//struct BroadcastAdmin;
