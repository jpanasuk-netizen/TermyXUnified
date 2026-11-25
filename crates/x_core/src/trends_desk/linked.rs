//! Open one X post from a pasted link: parse every common link form, read the
//! post with the free OpenCLI path, and keep its quoted post. A failed read is
//! an error message, never a made-up post.

use std::sync::atomic::AtomicBool;

use serde_json::Value;

use super::domain::{SourcePost, redact_secrets, status_from_url};
use super::fetch::{OpenCliDesk, SEARCH_TIMEOUT};
use super::media::{QuotedTweet, quoted_reference_id};
use super::parse::{ParseError, as_post_list, parse_json_blob};
use crate::publish::humanize_error;

/// The post a link points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRef {
    pub id: String,
    pub author: String,
    pub url: String,
}

const HOSTS: [&str; 6] = [
    "x.com/",
    "twitter.com/",
    "www.x.com/",
    "www.twitter.com/",
    "mobile.twitter.com/",
    "mobile.x.com/",
];

/// First X or Twitter status link in pasted text. Accepts x.com, twitter.com,
/// mobile.twitter.com, `?s=20` style params, `/i/status/ID`, `/i/web/status/ID`,
/// `/user/status/ID/photo/1`, links with no scheme, and links wrapped in
/// brackets, quotes or trailing punctuation.
pub fn parse_status_ref(input: &str) -> Option<StatusRef> {
    let flat = input.replace("/#!/", "/");
    for token in flat.split(|c: char| c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'')) {
        let token =
            token.trim_matches(|c: char| matches!(c, '(' | ')' | '[' | ']' | ',' | ';' | '.'));
        if token.is_empty() {
            continue;
        }
        let lower = token.to_ascii_lowercase();
        let full = if lower.starts_with("http://") || lower.starts_with("https://") {
            token.to_string()
        } else if HOSTS.iter().any(|host| lower.starts_with(host)) {
            format!("https://{token}")
        } else {
            continue;
        };
        if let Some(link) = status_from_url(&full) {
            return Some(StatusRef {
                id: link.id,
                author: link.author,
                url: link.canonical,
            });
        }
    }
    None
}

/// What the read returned for one post id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkedRead {
    pub post: SourcePost,
    /// Quoted post id the row named without embedding it.
    pub quoted_id: Option<String>,
}

/// The requested post out of one OpenCLI payload. The row must carry the
/// requested id: a thread read also returns replies, and a reply is not it.
pub fn linked_from_stdout(stdout: &str, id: &str) -> Result<LinkedRead, String> {
    let value = parse_json_blob(stdout).map_err(|error| error.to_string())?;
    let rows = as_post_list(value).map_err(|error| match error {
        ParseError::Provider(message) | ParseError::Malformed(message) => message,
    })?;
    let row = rows
        .iter()
        .find(|row| row_id(row) == Some(id))
        .ok_or_else(|| {
            format!(
                "OpenCLI did not return post {id}. It may be deleted, protected, or the OpenCLI session may be signed out."
            )
        })?;
    let post = SourcePost::from_value(row)
        .ok_or_else(|| format!("Post {id} came back with no readable text."))?;
    let quoted_id = if post.quoted.is_none() {
        quoted_reference_id(row)
    } else {
        None
    };
    Ok(LinkedRead { post, quoted_id })
}

fn row_id(row: &Value) -> Option<&str> {
    ["id", "id_str", "rest_id"]
        .iter()
        .find_map(|key| row.get(*key).and_then(Value::as_str))
}

pub fn status_args(id: &str) -> Vec<String> {
    vec![
        "twitter".into(),
        "thread".into(),
        id.to_string(),
        "--limit".into(),
        "1".into(),
        "-f".into(),
        "json".into(),
    ]
}

impl OpenCliDesk {
    /// Read the post a link names, with its quoted post when it has one.
    pub fn fetch_status(
        &self,
        link: &StatusRef,
        cancel: &AtomicBool,
    ) -> Result<SourcePost, String> {
        self.preflight()?;
        let read = self.read_status(&link.id, cancel)?;
        let mut post = read.post;
        if let Some(quoted_id) = read.quoted_id {
            post.quoted = Some(match self.read_status(&quoted_id, cancel) {
                Ok(inner) => QuotedTweet {
                    author: inner.post.author.clone(),
                    name: inner.post.author_name.clone(),
                    text: inner.post.text.clone(),
                    media: inner.post.media_items.clone(),
                    url: inner.post.url.clone(),
                },
                Err(_) => QuotedTweet {
                    url: format!("https://x.com/i/status/{quoted_id}"),
                    ..QuotedTweet::default()
                },
            });
        }
        Ok(post)
    }

    fn read_status(&self, id: &str, cancel: &AtomicBool) -> Result<LinkedRead, String> {
        let stdout = self
            .run(&status_args(id), SEARCH_TIMEOUT, cancel)
            .map_err(|error| shorten(&redact_secrets(&humanize_error(&error))))?;
        linked_from_stdout(&stdout, id)
    }
}

fn shorten(text: &str) -> String {
    text.chars().take(400).collect()
}

#[cfg(test)]
#[path = "linked_tests.rs"]
mod tests;
