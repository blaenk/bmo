use std::num;

use reqwest;
use serenity;

use util::html_to_markdown;

mod story;
mod comment;
mod item;
mod previewer;

pub use self::previewer::HackerNews;

pub const THUMBNAIL: &'static str = "https://news.ycombinator.com/y18.gif";
pub const ORANGE: u64 = 0xFF6600;

error_chain! {
    foreign_links {
        ParseInt(num::ParseIntError);
        Reqwest(reqwest::Error);
        Serenity(serenity::Error);
    }

    links {
        HtmlParse(html_to_markdown::Error, html_to_markdown::ErrorKind);
    }

    errors {
        InvalidHost {
            description("Host is not that of Hacker News (news.ycombinator.com)")
        }
        InvalidPath {
            description("URL path is not correct (/item)")
        }
        MissingId {
            description("URL is missing the id query parameter (id={id})")
        }
        TimeZoneParse(e: String) {
            description("Could not parse the IANA timezone identifier")
        }
    }
}
