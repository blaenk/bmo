use serenity::client::{Client, Context, EventHandler};
use serenity::model::{Ready, Message};

use slog::Logger;
use url::Url;
use linkify::{LinkFinder, LinkKind};

use preview::Preview;

// TODO
// Put this somewhere useful.
#[allow(dead_code)]
pub const EMOJI_POGCHAMP: &'static str = "<:pogchamp:281912440696864769>";

pub fn new_client(token: &str, bot: Bot) -> Client<Bot> {
    Client::new(token, bot)
}

pub struct Bot {
    log: Logger,
    previewers: Vec<Box<Preview>>,
}

impl Bot {
    pub fn new(log: Logger) -> Bot {
        info!(log, "creating Bot");

        Bot {
            log,
            previewers: vec![],
        }
    }

    pub fn push_previewer<T>(&mut self, previewer: T) where T: Preview + 'static {
        self.previewers.push(Box::new(previewer));
    }

    /// Scan the given `Message` for URLs and attempt to preview them.
    ///
    /// This doesn't return a `Result` because previewing is not something
    /// considered to be critical. If attempting to preview a given URL results
    /// in an `Error`, that error is logged and the URL is skipped.
    fn preview_links(&self, message: &Message, log: &Logger) {
        if message.is_own() {
            info!(log, "Ignoring own message");
            return;
        }

        let mut finder = LinkFinder::new();
        finder.kinds(&[LinkKind::Url]);

        for link in finder.links(&message.content) {
            let link = link.as_str();

            info!(log, "Detected link"; "link" => link);

            match Url::parse(link) {
                Ok(url) => {
                    // We could simply use `link` for our logging purposes but I
                    // would rather show it the way the parser shows it, in case
                    // it may point to any discrepancy in how we expect it to
                    // have been parsed.
                    let url_as_str = url.to_string();

                    let log = log.new(o!("url" => url_as_str));

                    info!(log, "Parsed URL");

                    for previewer in &self.previewers {
                        previewer.preview(&url, &message, &log);
                    }
                }
                Err(e) => {
                    error!(log, "Couldn't parse link as a URL";
                           "link" => link, "error" => e.to_string());
                }
            }
        }
    }
}

impl EventHandler for Bot {
    fn on_ready(&self, _context: Context, ready: Ready) {
        info!(self.log, "Connection established to gateway";
              "version" => ready.version, "session_id" => ready.session_id);
    }

    fn on_message(&self, _context: Context, message: Message) {
        let log = self.log.new(o!("message" => message.id.0));

        info!(log, "Received a message");

        self.preview_links(&message, &log);
    }
}
