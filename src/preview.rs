use url::Url;
use serenity::model::Message;
use slog::Logger;

// TODO
// Also pass the slog Logger as a parameter, which will already include all of
// the necessary information.
// NOTE
// I feel like the previewers system should instead work by being given a
// constructor for a type that implements Preview, so that each "request" is
// done from scratch?
// This probably isn't possible without impl Trait, and perhaps even then.
/// This trait represents a type that can preview a URL.
pub trait Preview {
    fn preview(&self, url: &Url, message: &Message, log: &Logger);
}
