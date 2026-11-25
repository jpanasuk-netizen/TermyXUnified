//! Reposts in OpenCLI-style rows: an embedded original, a bare `RT @handle:`
//! text, or a `retweeted_by` field. The card shows the original post with a
//! "reposted by" line. Nothing here invents an original.

use serde_json::Value;

use crate::model::strip_handle;

const ORIGINAL_KEYS: [&str; 4] = [
    "retweeted",
    "retweeted_status",
    "retweeted_tweet",
    "retweeted_status_result",
];

/// The embedded original of a repost row, unwrapped from GraphQL nesting.
pub(super) fn embedded_original(value: &Value) -> Option<&Value> {
    for key in ORIGINAL_KEYS {
        let Some(node) = value.get(key).filter(|item| item.is_object()) else {
            continue;
        };
        let inner = node.get("result").unwrap_or(node);
        return Some(inner.get("tweet").unwrap_or(inner));
    }
    None
}

/// `RT @handle: text` split into the original handle and the text after it.
/// A mention such as `RT @handle hello` has no colon and is not a repost.
pub(super) fn split_bare_repost(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("RT @")?;
    let handle: &str = rest.split(':').next()?;
    if handle.is_empty()
        || !handle
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        || !rest[handle.len()..].starts_with(':')
    {
        return None;
    }
    let body = rest[handle.len() + 1..].trim_start();
    (!body.is_empty()).then_some((handle, body))
}

/// A `retweeted_by` style field on a plain row.
pub(super) fn reposted_by_field(value: &Value) -> Option<String> {
    ["retweeted_by", "reposted_by"].iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .map(|text| strip_handle(text).to_string())
            .filter(|text| !text.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trends_desk::SourcePost;
    use serde_json::json;

    #[test]
    fn bare_rt_text_splits_and_a_mention_does_not() {
        assert_eq!(
            split_bare_repost("RT @alice_1: the real words"),
            Some(("alice_1", "the real words"))
        );
        assert_eq!(split_bare_repost("RT @alice hello"), None);
        assert_eq!(split_bare_repost("RT @alice:"), None);
        assert_eq!(split_bare_repost("hello RT @alice: x"), None);
    }

    #[test]
    fn an_embedded_original_shows_the_original_with_a_reposted_by_line() {
        let row = json!({
            "id": "900", "author": "reposter", "text": "RT @orig: cut off…",
            "url": "https://x.com/reposter/status/900",
            "retweeted_status": {
                "id": "100", "author": "orig", "text": "the full original text",
                "url": "https://x.com/orig/status/100", "likes": 7,
                "quoted_tweet": {"author": "third", "text": "inside", "url": "https://x.com/third/status/5"}
            }
        });
        let post = SourcePost::from_value(&row).unwrap();
        assert_eq!(post.author, "orig");
        assert_eq!(post.id, "100");
        assert_eq!(post.text, "the full original text");
        assert_eq!(post.reposted_by.as_deref(), Some("reposter"));
        assert_eq!(post.quoted.unwrap().author, "third");
    }

    #[test]
    fn graphql_nested_originals_unwrap() {
        let row = json!({
            "id": "900", "author": "reposter", "text": "RT @orig: x",
            "url": "https://x.com/reposter/status/900",
            "retweeted_status_result": {"result": {"tweet": {
                "id": "100", "author": "orig", "text": "nested original",
                "url": "https://x.com/orig/status/100"
            }}}
        });
        let post = SourcePost::from_value(&row).unwrap();
        assert_eq!(post.text, "nested original");
        assert_eq!(post.reposted_by.as_deref(), Some("reposter"));
    }

    #[test]
    fn a_bare_rt_row_shows_the_original_handle_and_text_never_made_up() {
        let row = json!({
            "id": "900", "author": "reposter", "text": "RT @orig: only what the row holds",
            "url": "https://x.com/reposter/status/900"
        });
        let post = SourcePost::from_value(&row).unwrap();
        assert_eq!(post.author, "orig");
        assert_eq!(post.text, "only what the row holds");
        assert_eq!(post.reposted_by.as_deref(), Some("reposter"));
        assert_eq!(post.id, "900");
    }

    #[test]
    fn a_retweeted_by_field_and_a_plain_post_behave() {
        let row = json!({
            "id": "5", "author": "orig", "text": "hello", "retweeted_by": "@fan",
            "url": "https://x.com/orig/status/5"
        });
        let post = SourcePost::from_value(&row).unwrap();
        assert_eq!(post.reposted_by.as_deref(), Some("fan"));
        assert_eq!(post.author, "orig");
        let plain =
            json!({"id": "6", "author": "a", "text": "hi", "url": "https://x.com/a/status/6"});
        assert!(
            SourcePost::from_value(&plain)
                .unwrap()
                .reposted_by
                .is_none()
        );
    }
}
