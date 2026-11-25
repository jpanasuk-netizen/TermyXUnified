//! Open an X post from a pasted link. The post loads through the free
//! OpenCLI read and shows as the same card as any desk post, with its quoted
//! post, so the take, hook and analysis tools work on it. A failed read shows
//! the error and the link, never made-up text. Nothing is posted.

use std::sync::Arc;

use gpui_kit::{
    ClipboardItem, Context, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, rgb,
};
use termy_x::io::SystemRunner;
use termy_x::trends_desk::{OpenCliDesk, SourcePost, WINDOW_PAD, parse_status_ref, wrap_token};

use crate::trends_desk::{TrendsDesk, button};

pub(crate) enum LinkedLoad {
    Loading { url: String },
    Ready { post: SourcePost },
    Failed { url: String, error: String },
}

impl TrendsDesk {
    /// Every post the window can act on: the columns plus the opened link.
    pub(crate) fn all_posts(&self) -> impl Iterator<Item = &SourcePost> + '_ {
        let linked = match &self.linked {
            Some(LinkedLoad::Ready { post }) => Some(post),
            _ => None,
        };
        self.columns
            .iter()
            .flat_map(|column| column.view.posts.iter())
            .chain(linked)
    }

    pub(crate) fn open_status_link(&mut self, raw: &str, cx: &mut Context<Self>) {
        let Some(link) = parse_status_ref(raw) else {
            self.action_note =
                "That is not an X post link. Paste an x.com or twitter.com status link.".into();
            cx.notify();
            return;
        };
        self.board = "link".into();
        self.selected = None;
        self.palette = false;
        self.palette_query.clear();
        self.linked = Some(LinkedLoad::Loading {
            url: link.url.clone(),
        });
        self.action_note = format!("Loading {}…", link.url);
        let config = self.config.clone();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    let desk = OpenCliDesk::new(&config, Arc::new(SystemRunner));
                    desk.fetch_status(&link, &cancel)
                        .map_err(|error| (link, error))
                })
                .await;
            let _ = this.update(cx, |desk, cx| {
                match result {
                    Ok(post) => {
                        desk.action_note =
                            format!("Opened @{}'s post. Nothing was posted.", post.author);
                        desk.selected = Some(format!("take:{}", post.id));
                        desk.linked = Some(LinkedLoad::Ready { post });
                    }
                    Err((link, error)) => {
                        desk.action_note = format!("Could not open the post: {error}");
                        desk.linked = Some(LinkedLoad::Failed {
                            url: link.url,
                            error,
                        });
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    fn clipboard_text(cx: &mut Context<Self>) -> Option<String> {
        let item: ClipboardItem = cx.read_from_clipboard()?;
        item.text().map(|text| text.to_string())
    }

    /// Ctrl+V with no field focused: a post link in the clipboard opens it.
    /// Anything else is left to the other paste handlers.
    pub(crate) fn paste_link_key(&mut self, cx: &mut Context<Self>) -> bool {
        match Self::clipboard_text(cx) {
            Some(text) if parse_status_ref(&text).is_some() => {
                self.open_status_link(&text, cx);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn palette_paste(&mut self, cx: &mut Context<Self>) {
        if let Some(text) = Self::clipboard_text(cx) {
            let line = text.lines().next().unwrap_or("").trim();
            self.palette_query
                .push_str(&line.chars().take(500).collect::<String>());
        }
    }

    pub(crate) fn palette_enter(&mut self, cx: &mut Context<Self>) {
        let query = self.palette_query.clone();
        if parse_status_ref(&query).is_some() {
            self.open_status_link(&query, cx);
        }
    }

    pub(crate) fn run_palette_cx(&mut self, action: &str, cx: &mut Context<Self>) {
        if action == "openlink" {
            let text = Self::clipboard_text(cx).unwrap_or_default();
            self.open_status_link(&text, cx);
        } else if let Some(raw) = action.strip_prefix("link:") {
            self.open_status_link(raw, cx);
        } else {
            self.run_palette(action);
        }
    }

    pub(crate) fn link_page(&self, width: f32, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut page = div()
            .id("desk-link-page")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .p(px(WINDOW_PAD))
            .child(button(
                "desk-link-back",
                "Back",
                false,
                cx.listener(|this, _, _, cx| {
                    this.board = "subject".into();
                    cx.stop_propagation();
                    cx.notify();
                }),
            ));
        page = match &self.linked {
            None => page.child(
                "No post link is open. Press Ctrl+V with a post link copied, or Ctrl+K and choose \"Open post link from clipboard\".",
            ),
            Some(LinkedLoad::Loading { url }) => page
                .child("Loading the post through OpenCLI…")
                .child(div().text_color(rgb(0x7af6ff)).child(wrap_token(url))),
            Some(LinkedLoad::Failed { url, error }) => page
                .child(
                    div()
                        .text_color(rgb(0xff6b6b))
                        .whitespace_normal()
                        .child(format!("Could not load this post. {error}")),
                )
                .child(div().text_color(rgb(0x7af6ff)).child(wrap_token(url))),
            Some(LinkedLoad::Ready { post }) => page.child(
                div()
                    .w(px((width - 2.0 * WINDOW_PAD).max(120.0)))
                    .min_w(px(0.0))
                    .child(self.tweet_card(post, &[], (width - 2.0 * WINDOW_PAD).max(120.0), cx)),
            ),
        };
        page.into_any_element()
    }
}
