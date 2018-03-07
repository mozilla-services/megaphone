use super::schema::versionv1;

#[derive(Debug, Queryable, Insertable)]
#[table_name = "versionv1"]
pub struct Version {
    pub service_id: String, // combination of the broadcast + '/' + collection ids
    pub version: String,    // version information
}
