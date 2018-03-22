use failure::ResultExt;
use diesel::{replace_into, RunQueryDsl};
use diesel::mysql::MysqlConnection;

use super::schema::broadcastsv1;
use error::{HandlerErrorKind, HandlerResult};

#[derive(Debug, Queryable, Insertable)]
#[table_name = "broadcastsv1"]
pub struct Broadcast {
    pub broadcaster_id: String,
    pub bchannel_id: String,
    pub version: String,
}

impl Broadcast {
    pub fn id(&self) -> String {
        format!("{}/{}", self.broadcaster_id, self.bchannel_id)
    }
}

/// An authorized broadcaster
pub struct Broadcaster {
    pub id: String,
}

impl Broadcaster {
    pub fn new_broadcast(
        self,
        conn: &MysqlConnection,
        bchannel_id: String,
        version: String,
    ) -> HandlerResult<usize> {
        let broadcast = Broadcast {
            broadcaster_id: self.id,
            bchannel_id: bchannel_id,
            version: version,
        };
        Ok(replace_into(broadcastsv1::table)
            .values(&broadcast)
            .execute(conn)
            .context(HandlerErrorKind::DBError)?)
    }
}

// An authorized reader of current broadcasts
//struct BroadcastAdmin;
