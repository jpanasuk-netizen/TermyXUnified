use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde_json::json;

use super::*;
use crate::config::XConfig;
use crate::io::ScriptedRunner;

fn desk(runner: ScriptedRunner) -> OpenCliDesk {
    let config = XConfig {
        opencli_bin: "opencli".into(),
        ..XConfig::default()
    };
    OpenCliDesk::new(&config, Arc::new(runner)).with_hermes_dir(None)
}

fn link(id: &str) -> StatusRef {
    StatusRef {
        id: id.into(),
        author: String::new(),
        url: format!("https://x.com/i/status/{id}"),
    }
}

#[test]
fn every_common_link_form_gives_the_same_post() {
    let forms = [
        "https://x.com/alice/status/123",
        "https://twitter.com/alice/status/123",
        "http://mobile.twitter.com/alice/status/123",
        "https://mobile.x.com/alice/status/123?s=20",
        "https://x.com/alice/status/123?s=46&t=abc#frag",
        "https://www.twitter.com/alice/status/123/",
        "https://x.com/alice/status/123/photo/1",
        "https://x.com/i/status/123",
        "https://x.com/i/web/status/123",
        "https://twitter.com/i/web/status/123?ref_src=twsrc",
        "x.com/alice/status/123",
        "twitter.com/alice/status/123?s=20",
        "look at this <https://x.com/alice/status/123>.",
        "(https://x.com/alice/status/123),",
        "https://twitter.com/#!/alice/status/123",
        "https://twitter.com/alice/statuses/123",
        "https://fxtwitter.com/alice/status/123",
    ];
    for form in forms {
        let got = parse_status_ref(form).unwrap_or_else(|| panic!("no link in {form}"));
        assert_eq!(got.id, "123", "{form}");
    }
    assert_eq!(
        parse_status_ref("https://x.com/alice/status/123")
            .unwrap()
            .url,
        "https://x.com/alice/status/123"
    );
    assert_eq!(
        parse_status_ref("https://x.com/i/status/123").unwrap().url,
        "https://x.com/i/status/123"
    );
}

#[test]
fn text_that_is_not_a_post_link_gives_none() {
    for text in [
        "",
        "hello",
        "https://x.com/alice",
        "https://x.com/alice/likes",
        "https://example.com/alice/status/123",
        "https://x.com/i/status/",
        "https://evil.com/?u=https://x.com/alice/status/123",
        "12345",
    ] {
        assert!(parse_status_ref(text).is_none(), "{text}");
    }
}

fn quote_row() -> serde_json::Value {
    json!({
        "id": "2119413665547821165",
        "author": "Jasper_Black",
        "text": "100 is enough",
        "likes": 1,
        "retweets": 0,
        "views": "5",
        "created_at": "Sun Nov 08 12:56:09 +0000 2026",
        "url": "https://x.com/Jasper_Black/status/2119413665547821165",
        "has_media": false,
        "media_urls": [],
        "quoted_tweet": {
            "id": "2118971448614334959",
            "author": "cryptoXRPwhale",
            "name": "XRP whale",
            "text": "IS OWNING 1,000 $XRP ENOUGH?",
            "url": "https://x.com/cryptoXRPwhale/status/2118971448614334959",
            "media_urls": ["https://pbs.twimg.com/media/abc.jpg"]
        }
    })
}

#[test]
fn a_quote_tweet_keeps_the_quoted_author_text_media_and_link() {
    let out = json!({"ok": true, "data": [quote_row()]}).to_string();
    let read = linked_from_stdout(&out, "2119413665547821165").unwrap();
    assert_eq!(read.post.author, "Jasper_Black");
    assert_eq!(read.post.text, "100 is enough");
    let quoted = read.post.quoted.expect("quoted post");
    assert_eq!(quoted.author, "cryptoXRPwhale");
    assert_eq!(quoted.name, "XRP whale");
    assert_eq!(quoted.text, "IS OWNING 1,000 $XRP ENOUGH?");
    assert_eq!(quoted.media.len(), 1);
    assert_eq!(
        quoted.url,
        "https://x.com/cryptoXRPwhale/status/2118971448614334959"
    );
    assert!(read.quoted_id.is_none());
}

#[test]
fn the_requested_row_wins_over_replies_and_a_missing_row_is_an_error() {
    let reply =
        json!({"id": "9", "author": "bob", "text": "a reply", "url": "https://x.com/bob/status/9"});
    let out = json!({"ok": true, "data": [reply, quote_row()]}).to_string();
    let read = linked_from_stdout(&out, "2119413665547821165").unwrap();
    assert_eq!(read.post.author, "Jasper_Black");
    let missing = linked_from_stdout(&out, "555").unwrap_err();
    assert!(missing.contains("did not return post 555"), "{missing}");
}

#[test]
fn failed_reads_say_so_and_never_return_a_post() {
    let cancel = AtomicBool::new(false);
    let runner = ScriptedRunner::new();
    runner.script(
        "opencli",
        &["twitter", "thread"],
        json!({"ok": false, "error": "tweet unavailable"}).to_string(),
    );
    let error = desk(runner).fetch_status(&link("77"), &cancel).unwrap_err();
    assert!(error.contains("tweet unavailable"), "{error}");

    let runner = ScriptedRunner::new();
    runner.script("opencli", &["twitter", "thread"], "not json");
    assert!(desk(runner).fetch_status(&link("77"), &cancel).is_err());

    let runner = ScriptedRunner::new();
    runner.script(
        "opencli",
        &["twitter", "thread"],
        json!({"ok": true, "data": []}).to_string(),
    );
    let error = desk(runner).fetch_status(&link("77"), &cancel).unwrap_err();
    assert!(error.contains("did not return post 77"), "{error}");

    let missing = desk(ScriptedRunner::new())
        .fetch_status(&link("77"), &cancel)
        .unwrap_err();
    assert!(missing.contains("opencli"), "{missing}");
}

#[test]
fn fetch_reads_one_post_with_one_thread_call_and_no_shell() {
    let runner = ScriptedRunner::new();
    runner.script(
        "opencli",
        &["twitter", "thread"],
        json!({"ok": true, "data": [quote_row()]}).to_string(),
    );
    let post = desk(runner.clone())
        .fetch_status(&link("2119413665547821165"), &AtomicBool::new(false))
        .unwrap();
    assert!(post.quoted.is_some());
    let calls = runner.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    let (_, args) = &calls[0];
    assert_eq!(&args[..3], ["twitter", "thread", "2119413665547821165"]);
    assert!(args.iter().all(|arg| arg != "sh" && arg != "x.exe"));
}

#[test]
fn a_quote_named_only_by_id_is_fetched_and_a_failed_fetch_keeps_a_link() {
    let mut row = quote_row();
    row.as_object_mut().unwrap().remove("quoted_tweet");
    row["quoted_status_id_str"] = json!("2118971448614334959");
    let inner = json!({
        "id": "2118971448614334959", "author": "cryptoXRPwhale",
        "text": "the original", "url": "https://x.com/cryptoXRPwhale/status/2118971448614334959"
    });
    let runner = ScriptedRunner::new();
    runner.script(
        "opencli",
        &["twitter", "thread", "2119413665547821165"],
        json!({"ok": true, "data": [row.clone()]}).to_string(),
    );
    runner.script(
        "opencli",
        &["twitter", "thread", "2118971448614334959"],
        json!({"ok": true, "data": [inner]}).to_string(),
    );
    let post = desk(runner)
        .fetch_status(&link("2119413665547821165"), &AtomicBool::new(false))
        .unwrap();
    let quoted = post.quoted.unwrap();
    assert_eq!(quoted.text, "the original");
    assert_eq!(quoted.author, "cryptoXRPwhale");

    let runner = ScriptedRunner::new();
    runner.script(
        "opencli",
        &["twitter", "thread", "2119413665547821165"],
        json!({"ok": true, "data": [row]}).to_string(),
    );
    runner.script(
        "opencli",
        &["twitter", "thread", "2118971448614334959"],
        json!({"ok": false, "error": "gone"}).to_string(),
    );
    let post = desk(runner)
        .fetch_status(&link("2119413665547821165"), &AtomicBool::new(false))
        .unwrap();
    let quoted = post.quoted.unwrap();
    assert!(quoted.text.is_empty() && quoted.media.is_empty());
    assert_eq!(quoted.url, "https://x.com/i/status/2118971448614334959");
}
