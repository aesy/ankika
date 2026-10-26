use anki::error::AnkiError;
use anki::prelude::I18n;
use reqwest::Client;

/// Same client setup as Anki desktop uses for syncing.
pub fn http_client() -> Result<Client, reqwest::Error> {
    Client::builder().http1_only().build()
}

/// Anki's `Display` only names the variant; its message has the details.
pub fn english(error: &AnkiError) -> String {
    error.message(&I18n::template_only())
}
