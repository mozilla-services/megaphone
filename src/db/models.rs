use super::schema::versionv1;
use super::super::http::Broadcaster;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "versionv1"]
pub struct Version {
    pub service_id: String, // combination of the broadcast + '/' + collection ids
    pub version: String,    // version information
}

impl Version {
    pub fn new(broadcaster: Broadcaster, collection_id: String, version: String) -> Version {
        Version {
            service_id: format!("{}/{}", broadcaster.id, collection_id),
            version: version,
        }
    }
}
