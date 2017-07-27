use super::{ErrorKind, Result};

use serenity::model::{Channel, Message};
use serenity::utils::MessageBuilder;

use url::Url;

use slog::Logger;

use preview::Preview;

use super::item::Item;
use super::story::Story;
use super::comment::Comment;

pub fn channel_from_message(message: &Message) -> Result<Channel> {
    match message.channel_id.get() {
        Ok(channel) => Ok(channel),
        Err(e) => bail!(e),
    }
}

pub struct HackerNews;

impl HackerNews {
    pub fn preview_story(&self, message: &Message, story: Story, log: &Logger) -> Result<()> {
        let channel = channel_from_message(message)?;

        let description = format!(
            "**{}** points. **{}** comments",
            story.score(),
            story.kids().len()
        );

        // TODO
        // Also send story body in the case of self-posts, as with comment?

        channel
            .send_message(|m| {
                m.embed(|e| {
                    e.url(&story.url())
                        .title(story.title())
                        .description(&description)
                        .color(super::ORANGE)
                        .timestamp(&story.local_time())
                        .footer(|f| f.icon_url(super::THUMBNAIL).text("Hacker News"))
                })
            })
            .map(|message| {
                info!(log, "Sent message"; "id" => message.id.0);
            })
            .map_err(From::from)
    }

    pub fn preview_comment(&self, message: &Message, comment: Comment, log: &Logger) -> Result<()> {
        let channel = channel_from_message(message)?;

        let log = log.new(o!("comment_id" => comment.id()));

        let description = if comment.kids().is_empty() {
            MessageBuilder::new()
                .push("by ")
                .push_bold(comment.by())
                .build()
        } else {
            MessageBuilder::new()
                .push_bold(&comment.kids().len().to_string())
                .push(" replies. by ")
                .push_bold(comment.by())
                .build()
        };

        let title = match comment.get_story() {
            Ok(story) => format!("Comment on: {}", story.title()),
            Err(e) => {
                error!(log, "Couldn't get comment's story"; "error" => e.to_string());
                bail!(e);
            }
        };

        channel
            .send_message(|m| {
                m.embed(|e| {
                    e.url(&comment.url())
                        .title(&title)
                        .description(&description)
                        .color(super::ORANGE)
                        .timestamp(&comment.local_time())
                        .footer(|f| f.icon_url(super::THUMBNAIL).text("Hacker News"))
                })
            })
            .map(|message| {
                info!(log, "Sent HN comment embed"; "id" => message.id.0);
            })?;

        let body = comment.text_as_markdown()?;

        let comment = MessageBuilder::new()
            .push(":speech_left: ")
            .push_bold("BEGIN QUOTE")
            .push(" :speech_balloon:\n")
            .push(&body)
            .push("\n")
            .push(":speech_left: ")
            .push_bold("END QUOTE")
            .push(" :speech_balloon:")
            .build();

        channel
            .say(&comment)
            .map(|message| {
                info!(log, "Sent HN comment body"; "id" => message.id.0);
            })
            .map_err(From::from)
    }
}

impl Preview for HackerNews {
    fn preview(&self, url: &Url, message: &Message, log: &Logger) {
        let result = Item::from_url(&url).and_then(|item| match item {
            Item::Story(story) => self.preview_story(message, story, log),
            Item::Comment(comment) => self.preview_comment(message, comment, log),
        });

        match result {
            Ok(()) => {
                info!(log, "Previewed HN URL");
            }
            Err(e) => {
                match *e.kind() {
                    ErrorKind::InvalidHost => {
                        info!(log, "Ignoring non-HN URL");
                    }
                    _ => {
                        error!(log, "Couldn't preview HN URL"; "error" => e.to_string());
                    }
                }
            }
        }
    }
}

