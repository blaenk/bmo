use chrono::{DateTime, Utc, Local};
use chrono::serde::ts_seconds;
use chrono_tz::Tz;

use super::{Error, ErrorKind, Result};

use super::story::Story;
use super::item::Item;

use util;

#[derive(Clone, Debug, Deserialize)]
pub struct Comment {
    id: i64,
    by: String,

    #[serde(default)]
    kids: Vec<i64>,

    text: String,

    #[serde(with = "ts_seconds")]
    time: DateTime<Utc>,

    parent: i64,
}

impl Comment {
    /// Find the root ancestor of this item.
    ///
    /// This only really applies to `Item::Comment`s, hence the `Option`.
    pub fn get_story(&self) -> Result<Story> {
        let parent = match Item::get(self.parent) {
            Ok(parent) => parent,
            Err(e) => return Err(e),
        };

        match parent {
            Item::Story(story) => Ok(story),
            Item::Comment(comment) => comment.get_story(),
        }
    }

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

    pub fn url(&self) -> String {
        format!("https://news.ycombinator.com/item?id={}", self.id())
    }

    pub fn author_url(&self) -> String {
        format!("https://news.ycombinator.com/user?id={}", self.by())
    }
}
