use std::collections::HashMap;

use diesel::mysql::MysqlConnection;
use diesel::sql_types::Text;
use diesel::{sql_query, QueryDsl, RunQueryDsl};
use failure::ResultExt;

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
        Broadcaster { id }
    }

    /// Broadcast a new version
    ///
    /// Returns:
    ///
    /// Err(HandlerError) on failure.
    ///
    /// Ok(true) if this Broadcast did not have a current version and one was
    /// successfully created.
    ///
    /// Ok(false) if this Broadcast had an existing version that was
    /// successfully modified to the new version.
    pub fn broadcast_new_version(
        self,
        conn: &MysqlConnection,
        bchannel_id: &str,
        version: &str,
    ) -> HandlerResult<bool> {
        let affected_rows = sql_query(include_str!("upsert_broadcast.sql"))
            .bind::<Text, _>(&self.id)
            .bind::<Text, _>(bchannel_id)
            .bind::<Text, _>(version)
            .bind::<Text, _>(version)
            .execute(conn)
            .context(HandlerErrorKind::DBError)?;
        Ok(affected_rows == 1)
    }
}

/// An authorized reader of broadcasts
pub struct Reader {
    pub id: String,
}

impl Reader {
    pub fn new(id: String) -> Reader {
        Reader { id }
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
