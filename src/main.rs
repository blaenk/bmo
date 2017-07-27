extern crate dotenv;
extern crate ctrlc;

extern crate chrono;
extern crate chrono_tz;

extern crate url;
extern crate linkify;

#[macro_use]
extern crate html5ever;
extern crate reqwest;

#[macro_use]
extern crate slog;

extern crate slog_async;
extern crate slog_envlogger;
extern crate slog_scope;
extern crate slog_stdlog;
extern crate slog_term;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate serenity;

#[macro_use]
extern crate error_chain;

mod preview;
mod bot;
mod hacker_news;
mod util;

mod errors {
    error_chain! {
        foreign_links {
            Serenity(::serenity::Error);
            Reqwest(::reqwest::Error);
        }

        links {
            HackerNews(::hacker_news::Error, ::hacker_news::ErrorKind);
        }
    }
}

use std::env;

use slog::Drain;

use bot::Bot;

fn main() {
    dotenv::dotenv().ok();

    let decorator = slog_term::TermDecorator::new().stderr().build();
    let formatter = slog_term::CompactFormat::new(decorator).build().fuse();
    let logger = slog_envlogger::new(formatter);
    let drain = slog_async::Async::default(logger);

    let root_logger = slog::Logger::root(
        drain.fuse(),
        o!(
            "version" => env!("CARGO_PKG_VERSION"),
            // NOTE
            // Uncomment this to get SLOC location
            // "place" => slog::FnValue(move |info| {
            //     format!("{}:{}", info.file(), info.line())
            // })
        ),
    );

    let _global_logger_guard =
        slog_stdlog::init().expect("Couldn't initialize global slog-stdlog logger.");

    slog_scope::scope(&root_logger, || {
        // Create client.
        let token = env::var("DISCORD_TOKEN").expect("token");

        let mut bot = Bot::new(root_logger.new(o!("scope" => "Bot")));

        bot.push_previewer(hacker_news::HackerNews);

        let mut client = bot::new_client(&token, bot);

        // Listen for signal.
        let closer = client.close_handle();

        let ctrlc_logger = root_logger.clone();

        ctrlc::set_handler(move || {
            info!(ctrlc_logger, "Received termination signal. Terminating.");

            closer.close();
        }).expect("Error setting handler.");

        // Start client.
        if let Err(e) = client.start_autosharded() {
            match e {
                serenity::Error::Client(serenity::client::ClientError::Shutdown) => {
                    info!(root_logger, "Shutting down.")
                }
                _ => error!(root_logger, "Problem with starting the client."; "error" => e.to_string()),
            }
        }
    });
}
