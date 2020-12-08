use rocket::{
    request::{self, FromRequest},
    Config, Outcome, Request, State,
};
use serde::{
    ser::{SerializeMap, Serializer},
    Serialize,
};
use std::collections::{BTreeMap, HashMap};

use crate::error::Result;

#[derive(Clone, Debug)]
pub struct Tags {
    pub tags: HashMap<String, String>,
    pub extra: HashMap<String, String>,
}

impl Default for Tags {
    fn default() -> Tags {
        Tags {
            tags: HashMap::new(),
            extra: HashMap::new(),
        }
    }
}

impl Serialize for Tags {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_map(Some(self.tags.len()))?;
        for tag in self.tags.clone() {
            if !tag.1.is_empty() {
                seq.serialize_entry(&tag.0, &tag.1)?;
            }
        }
        seq.end()
    }
}

// Tags are extra data to be recorded in metric and logging calls.
// If additional tags are required or desired, you will need to add them to the
// mutable extensions, e.g.
// ```
//      let mut tags = request.extensions_mut().get::<Tags>();
//      tags.insert("SomeLabel".to_owned(), "whatever".to_owned());
// ```
// how you get the request (or the response, and it's set of `extensions`) to whatever
// function requires it, is left as an exercise for the reader.
impl Tags {
    /*
    pub fn with_tags(tags: HashMap<String, String>) -> Tags {
        if tags.is_empty() {
            return Tags::default();
        }
        Tags {
            tags,
            extra: HashMap::new(),
        }
    }

    pub fn get(&self, label: &str) -> String {
        let none = "None".to_owned();
        self.tags.get(label).map(String::from).unwrap_or(none)
    }
    */

    pub fn extend(&mut self, tags: HashMap<String, String>) {
        self.tags.extend(tags);
    }

    /* // reserved for KV impl
    pub fn tag_tree(self) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();

        for (k, v) in self.tags {
            result.insert(k.clone(), v.clone());
        }
        result
    }

    pub fn extra_tree(self) -> BTreeMap<String, Value> {
        let mut result = BTreeMap::new();

        for (k, v) in self.extra {
            result.insert(k.clone(), Value::from(v));
        }
        result
    }
    */
}

impl<'a, 'r> FromRequest<'a, 'r> for Tags {
    type Error = failure::Error;

    fn from_request(req: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(req.guard::<State<Tags>>().unwrap().inner().clone())
    }
}

impl Tags {
    pub fn init(_config: &Config) -> Result<Self> {
        let tags = HashMap::new();
        let extra = HashMap::new();
        /* parse the header?
        if let Some(ua) = req.headers().get("User-Agent") {
            if let Ok(uas) = ua.to_str() {
                let (ua_result, metrics_os, metrics_browser) = parse_user_agent(uas);
                insert_if_not_empty("ua.os.family", metrics_os, &mut tags);
                insert_if_not_empty("ua.browser.family", metrics_browser, &mut tags);
                insert_if_not_empty("ua.name", ua_result.name, &mut tags);
                insert_if_not_empty("ua.os.ver", &ua_result.os_version.to_owned(), &mut tags);
                insert_if_not_empty("ua.browser.ver", ua_result.version, &mut tags);
                extra.insert("ua".to_owned(), uas.to_string());
            }
        }
        tags.insert("uri.method".to_owned(), req_head.method.to_string());
        // `uri.path` causes too much cardinality for influx but keep it in
        // extra for sentry
        extra.insert("uri.path".to_owned(), req_head.uri.to_string());
        */
        Ok(Tags { tags, extra })
    }
}

impl Into<BTreeMap<String, String>> for Tags {
    fn into(self) -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();

        for (k, v) in self.tags {
            result.insert(k.clone(), v.clone());
        }

        result
    }
}

/*
impl KV for Tags {
    fn serialize(&self, _rec: &Record<'_>, serializer: &mut dyn slog::Serializer) -> slog::Result {
        for (key, val) in &self.tags {
            let k = Key::from(key.clone());     // Key::from wants a static.
            serializer.emit_str(k.as_ref(), &val)?;
        }
        Ok(())
    }
}
*/
