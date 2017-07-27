use reqwest;
use url::Url;

use super::{Error, ErrorKind, Result, ResultExt};

use super::comment::Comment;
use super::story::Story;

// API Reference: https://github.com/HackerNews/API
//
// Stories may have text fields, such as when they are Ask HN posts. Instead of
// making things tedious by wrapping it in an Option, I tell Serde to make it an
// empty string in the event that the field isn't present. This is because the
// HN Firebase API entirely omits the field when it's not relevant. The same is
// done for comments with no kids (i.e. replies).

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Item {
    Comment(Comment),
    Story(Story),
}

impl Item {
    pub fn get(id: i64) -> Result<Item> {
        let url = format!("https://hacker-news.firebaseio.com/v0/item/{}.json", id);

        reqwest::get(&url)?
            .json::<Item>()
            .chain_err(|| "Couldn't GET HN API endpoint")
    }

    // https://news.ycombinator.com/item?id=14817557
    pub fn from_url(url: &Url) -> Result<Item> {
        if let Some(host) = url.host_str() {
            ensure!(host == "news.ycombinator.com", ErrorKind::InvalidHost);
        } else {
            bail!(ErrorKind::InvalidHost);
        }

        ensure!(url.path() == "/item", ErrorKind::InvalidPath);

        url.query_pairs()
            .find(|param| param.0 == "id")
            .ok_or(Error::from_kind(ErrorKind::MissingId))
            .and_then(|param| {
                param
                    .1
                    .parse::<i64>()
                    .map_err(|e| Error::from_kind(ErrorKind::ParseInt(e)))
                    .chain_err(|| "Couldn't parse URL id as an i64 integer")
            })
            .and_then(Item::get)
    }

    pub fn comment(self) -> Option<Comment> {
        match self {
            Item::Comment(comment) => Some(comment),
            Item::Story(..) => None,
        }
    }

    pub fn story(self) -> Option<Story> {
        match self {
            Item::Story(story) => Some(story),
            Item::Comment(..) => None,
        }
    }

    pub fn is_comment(&self) -> bool {
        if let Item::Comment { .. } = *self {
            true
        } else {
            false
        }
    }

    pub fn is_story(&self) -> bool {
        if let Item::Story { .. } = *self {
            true
        } else {
            false
        }
    }
}

// TODO
// To test this stuff we could use reqwest_mock's StubClient
// https://docs.rs/reqwest_mock/0.2.0/reqwest_mock/client/struct.StubClient.html
//
// It would be cool if:
//
//   * IntoBody was implemented for File
//   * it could be setup to redirect all requests to files on the fs, i.e.
//     `somesite.com/some/resource/123` would map to
//     `tests/fixtures/somesite.com/some/resource/123`

#[test]
fn test_item_parse() {
    use chrono::{NaiveDateTime, DateTime, Utc};
    use std::path::Path;

    let _fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hacker_news/13028891.json");

    let item = Item::get(13028891).expect("Couldn't get the item");
    let comment = item.comment().expect("Not a comment!");

    assert_eq!(13028891, comment.id());
    assert_eq!("filereaper", comment.by());
    assert_eq!(
        "https://news.ycombinator.com/item?id=13028891",
        comment.url()
    );
    assert_eq!(
        "https://news.ycombinator.com/user?id=filereaper",
        comment.author_url()
    );
    assert_eq!(&[13029453, 13028998, 13029094, 13029028], comment.kids());

    let timestamp = NaiveDateTime::from_timestamp(1479973426, 0);
    let expected_time = DateTime::<Utc>::from_utc(timestamp, Utc);

    assert_eq!(expected_time, comment.time());
}

#[test]
fn test_get_story() {
    let deep = Item::get(14775602).expect("Couldn't get item");
    let root = deep.comment()
        .expect("Not a comment!")
        .get_story()
        .expect("Item didn't have a parent");

    assert_eq!(14774167, root.id());
}
