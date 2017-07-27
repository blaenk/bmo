use chrono::{DateTime, Utc, Local};
use chrono::serde::ts_seconds;
use chrono_tz::Tz;

use util;

use super::{Error, ErrorKind, Result};

#[derive(Clone, Debug, Deserialize)]
pub struct Story {
    id: i64,
    by: String,

    #[serde(default)]
    kids: Vec<i64>,

    #[serde(default)]
    text: String,

    #[serde(with = "ts_seconds")]
    time: DateTime<Utc>,

    title: String,
    url: String,
    score: i64,
    descendants: i64,
}

impl Story {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn by(&self) -> &str {
        &self.by
    }

    pub fn kids(&self) -> &[i64] {
        &self.kids
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn text_as_markdown(&self) -> Result<String> {
        util::html_to_markdown::convert(&self.text).map_err(From::from)
    }

    pub fn time(&self) -> DateTime<Utc> {
        self.time
    }

    pub fn local_time(&self) -> DateTime<Local> {
        self.time.with_timezone(&Local)
    }

    /// The time, formatted for a particular timezone expressed as an IANA
    /// timezone identifier.
    pub fn time_with_timezone(&self, timezone: &str) -> Result<DateTime<Tz>> {
        let tz: Tz = timezone
            .parse()
            .map_err(|e| Error::from_kind(ErrorKind::TimeZoneParse(e)))?;

        Ok(self.time().with_timezone(&tz))
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn url(&self) -> String {
        format!("https://news.ycombinator.com/item?id={}", self.id())
    }

    pub fn score(&self) -> i64 {
        self.score
    }

    pub fn descendants(&self) -> i64 {
        self.descendants
    }

    pub fn author_url(&self) -> String {
        format!("https://news.ycombinator.com/user?id={}", self.by())
    }
}
