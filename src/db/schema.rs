#![allow(proc_macro_derive_resolution_fallback)]

table! {
    broadcastsv1 (broadcaster_id, bchannel_id) {
        broadcaster_id -> Varchar,
        bchannel_id -> Varchar,
        last_updated -> Timestamp,
        version -> Varchar,
    }
}
