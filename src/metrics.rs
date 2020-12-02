use std::net::UdpSocket;
use std::time::Instant;

use cadence::{
    BufferedUdpMetricSink, Counted, Metric, NopMetricSink, QueuingMetricSink, StatsdClient,
    StatsdClientBuilder, Timed,
};
use rocket::{
    config::ConfigError,
    request::{self, FromRequest},
    Config, Outcome, Request, State,
};
use slog::{error, trace, warn, Logger};

use crate::error;
use crate::logging;
use crate::tags::Tags;

#[derive(Debug, Clone)]
pub struct MetricTimer {
    pub label: String,
    pub start: Instant,
    // pub tags: Tags,
}

#[derive(Debug, Clone)]
pub struct Metrics {
    client: Option<StatsdClient>,
    tags: Option<Tags>,
    log: Logger,
    timer: Option<MetricTimer>,
}

impl Drop for Metrics {
    fn drop(&mut self) {
        //let tags = self.tags.clone().unwrap_or_default();
        if let Some(client) = self.client.as_ref() {
            if let Some(timer) = self.timer.as_ref() {
                let lapse = (Instant::now() - timer.start).as_millis() as u64;
                warn!(
                    self.log,
                    "⌚ Ending timer at nanos: {:?} : {:?}", &timer.label, lapse
                );
                let tagged = client.time_with_tags(&timer.label, lapse);
                // Include any "hard coded" tags.
                // tagged = tagged.with_tag("version", env!("CARGO_PKG_VERSION"));
                // let tags = timer.tags.tags.clone();
                /*
                let keys = tags.keys();
                for tag in keys {
                    tagged = tagged.with_tag(tag, &tags.get(tag).unwrap())
                }
                */
                match tagged.try_send() {
                    Err(e) => {
                        // eat the metric, but log the error
                        warn!(self.log, "⚠️ Metric {} error: {:?} ", &timer.label, e);
                    }
                    Ok(v) => {
                        trace!(self.log, "⌚ {:?}", v.as_metric_str());
                    }
                }
            }
        }
    }
}

impl Metrics {
    pub fn sink() -> StatsdClientBuilder {
        StatsdClient::builder("", NopMetricSink)
    }

    pub fn init(config: &Config) -> error::Result<Metrics> {
        let logging = logging::init_logging(config)?;
        let builder = match config.get_string("statsd_host") {
            Ok(statsd_host) => {
                let socket = UdpSocket::bind("0.0.0.0:0")?;
                socket.set_nonblocking(true)?;

                let host = (
                    statsd_host.as_str(),
                    config.get_int("statsd_port").unwrap_or(8125) as u16,
                );
                let udp_sink = BufferedUdpMetricSink::from(host, socket)?;
                let sink = QueuingMetricSink::from(udp_sink);
                StatsdClient::builder(
                    &config
                        .get_string("statsd_label")
                        .unwrap_or("megaphone".to_string()),
                    sink,
                )
            }
            Err(ConfigError::Missing(_)) => Self::sink(),
            Err(e) => {
                error!(logging, "Could not build metric: {:?}", e);
                Err(error::HandlerErrorKind::InternalError)?
            }
        };
        Ok(Metrics {
            client: Some(
                builder
                    .with_error_handler(|err| println!("Metric send error: {:?}", err))
                    .build(),
            ),
            log: logging.clone(),
            timer: None,
            tags: Some(Tags::init(config)?),
        })
    }

    // increment a counter with no tags data.
    pub fn incr(&self, label: &str) {
        self.incr_with_tags(label, None)
    }

    pub fn incr_with_tags(&self, label: &str, tags: Option<Tags>) {
        if let Some(client) = self.client.as_ref() {
            let mut tagged = client.incr_with_tags(label);
            let mut mtags = self.tags.clone().unwrap_or_default();
            if let Some(tags) = tags {
                mtags.extend(tags.tags);
            }
            for key in mtags.tags.keys().clone() {
                if let Some(val) = mtags.tags.get(key) {
                    tagged = tagged.with_tag(&key, val.as_ref());
                }
            }
            // Include any "hard coded" tags.
            // incr = incr.with_tag("version", env!("CARGO_PKG_VERSION"));
            match tagged.try_send() {
                Err(e) => {
                    // eat the metric, but log the error
                    warn!(self.log, "⚠️ Metric {} error: {:?} ", label, e);
                }
                Ok(v) => trace!(self.log, "☑️ {:?}", v.as_metric_str()),
            }
        }
    }

    pub fn timer_with_tags(&self, label: &str, lapse: u64, tags: Option<Tags>) {
        if let Some(client) = self.client.as_ref() {
            let mut tagged = client.time_with_tags(label, lapse);
            let mtags = tags.unwrap_or_default();
            for key in mtags.tags.keys().clone() {
                if let Some(val) = mtags.tags.get(key) {
                    tagged = tagged.with_tag(&key, val.as_ref());
                }
            }
            match tagged.try_send() {
                Err(e) => {
                    warn!(self.log, "Metric {} error {:?}", label, e);
                }
                Ok(v) => {
                    dbg!(v);
                }
            }
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Metrics {
    type Error = failure::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(
            request
                .guard::<State<'_, Metrics>>()
                .unwrap()
                .inner()
                .clone(),
        )
    }
}
