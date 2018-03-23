use std::collections::HashMap;

use failure::ResultExt;
use diesel::{replace_into, QueryDsl, RunQueryDsl};
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
    pub fn new(id: String) -> Broadcaster {
        Broadcaster { id: id }
    }

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

/// An authorized reader of the broadcasts
pub struct Reader {
    pub id: String,
}

impl Reader {
    pub fn new(id: String) -> Reader {
        Reader { id: id }
    }

    pub fn read_broadcasts(
        &self,
        conn: &MysqlConnection,
    ) -> HandlerResult<HashMap<String, String>> {
        // flatten into HashMap FromIterator<(K, V)>
        Ok(broadcastsv1::table
            .select((
                broadcastsv1::broadcaster_id,
                broadcastsv1::bchannel_id,
                broadcastsv1::version,
            ))
            .load::<Broadcast>(conn)
            .context(HandlerErrorKind::DBError)?
            .into_iter()
            .map(|bcast| (bcast.id(), bcast.version))
            .collect())
    }
}
