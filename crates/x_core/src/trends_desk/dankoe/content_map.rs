//! Dan Koe step 1: Point A, Point B, 2-3 topics, 3-5 subtopics each and
//! 3-5 post ideas per subtopic. Drafts can be tagged to one subtopic so every
//! post is a step from A to B. Saved in the Termy XL data folder only.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::Draft;

pub const CONTENT_MAP_FILE: &str = "content_map.json";
const MAX_TOPICS: usize = 5;
const MAX_SUBTOPICS: usize = 7;
const MAX_IDEAS: usize = 7;
const SEP: &str = " › ";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapIdea {
    pub text: String,
    #[serde(default)]
    pub draft_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapSubtopic {
    pub name: String,
    #[serde(default)]
    pub ideas: Vec<MapIdea>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapTopic {
    pub name: String,
    #[serde(default)]
    pub subtopics: Vec<MapSubtopic>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentMap {
    #[serde(default)]
    pub point_a: String,
    #[serde(default)]
    pub point_b: String,
    #[serde(default)]
    pub topics: Vec<MapTopic>,
    /// Draft id -> "Topic › Subtopic".
    #[serde(default)]
    pub tags: BTreeMap<String, String>,
}

/// Which topic and subtopic the map panel has selected.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MapPick {
    pub topic: Option<usize>,
    pub sub: Option<usize>,
}

fn clean(text: &str) -> Result<String, String> {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        Err("Type something in the New item box first.".into())
    } else {
        Ok(text)
    }
}

fn same(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

impl ContentMap {
    pub fn add_topic(&mut self, name: &str) -> Result<usize, String> {
        let name = clean(name)?;
        if self.topics.iter().any(|topic| same(&topic.name, &name)) {
            return Err(format!("Topic \"{name}\" is already on the map."));
        }
        if self.topics.len() >= MAX_TOPICS {
            return Err(format!(
                "The map holds {MAX_TOPICS} topics. Dan Koe's range is 2-3."
            ));
        }
        self.topics.push(MapTopic {
            name,
            subtopics: Vec::new(),
        });
        Ok(self.topics.len() - 1)
    }

    pub fn add_subtopic(&mut self, topic: usize, name: &str) -> Result<usize, String> {
        let name = clean(name)?;
        let Some(item) = self.topics.get_mut(topic) else {
            return Err("Pick a topic first.".into());
        };
        if item.subtopics.iter().any(|sub| same(&sub.name, &name)) {
            return Err(format!(
                "Subtopic \"{name}\" is already under {}.",
                item.name
            ));
        }
        if item.subtopics.len() >= MAX_SUBTOPICS {
            return Err(format!(
                "{} already has {MAX_SUBTOPICS} subtopics. The range is 3-5.",
                item.name
            ));
        }
        item.subtopics.push(MapSubtopic {
            name,
            ideas: Vec::new(),
        });
        Ok(item.subtopics.len() - 1)
    }

    pub fn add_idea(&mut self, topic: usize, sub: usize, text: &str) -> Result<usize, String> {
        let text = clean(text)?;
        let Some(item) = self
            .topics
            .get_mut(topic)
            .and_then(|topic| topic.subtopics.get_mut(sub))
        else {
            return Err("Pick a subtopic first.".into());
        };
        if item.ideas.iter().any(|idea| same(&idea.text, &text)) {
            return Err("That idea is already there.".into());
        }
        if item.ideas.len() >= MAX_IDEAS {
            return Err(format!(
                "{} already has {MAX_IDEAS} ideas. The range is 3-5.",
                item.name
            ));
        }
        item.ideas.push(MapIdea {
            text,
            draft_id: None,
        });
        Ok(item.ideas.len() - 1)
    }

    pub fn remove_topic(&mut self, topic: usize) -> bool {
        if topic >= self.topics.len() {
            return false;
        }
        let name = self.topics.remove(topic).name;
        let prefix = format!("{name}{SEP}");
        self.tags.retain(|_, path| !path.starts_with(&prefix));
        true
    }

    pub fn remove_subtopic(&mut self, topic: usize, sub: usize) -> bool {
        let Some(item) = self.topics.get_mut(topic) else {
            return false;
        };
        if sub >= item.subtopics.len() {
            return false;
        }
        let path = format!("{}{SEP}{}", item.name, item.subtopics.remove(sub).name);
        self.tags.retain(|_, tag| tag != &path);
        true
    }

    pub fn remove_idea(&mut self, topic: usize, sub: usize, idea: usize) -> bool {
        match self
            .topics
            .get_mut(topic)
            .and_then(|topic| topic.subtopics.get_mut(sub))
        {
            Some(item) if idea < item.ideas.len() => {
                item.ideas.remove(idea);
                true
            }
            _ => false,
        }
    }

    pub fn path(&self, topic: usize, sub: usize) -> Option<String> {
        let topic = self.topics.get(topic)?;
        let sub = topic.subtopics.get(sub)?;
        Some(format!("{}{SEP}{}", topic.name, sub.name))
    }

    /// Every "Topic › Subtopic" in map order.
    pub fn subtopic_paths(&self) -> Vec<String> {
        self.topics
            .iter()
            .flat_map(|topic| {
                topic
                    .subtopics
                    .iter()
                    .map(move |sub| format!("{}{SEP}{}", topic.name, sub.name))
            })
            .collect()
    }

    /// The tag after `current`, then no tag, then the first one again.
    pub fn next_tag(&self, current: Option<&str>) -> Option<String> {
        let paths = self.subtopic_paths();
        match current.and_then(|now| paths.iter().position(|path| path == now)) {
            None => paths.first().cloned(),
            Some(index) => paths.get(index + 1).cloned(),
        }
    }

    pub fn tag_for(&self, draft_id: &str) -> Option<&str> {
        self.tags.get(draft_id).map(String::as_str)
    }

    pub fn set_tag(&mut self, draft_id: &str, tag: Option<String>) {
        match tag {
            Some(tag) => {
                self.tags.insert(draft_id.to_string(), tag);
            }
            None => {
                self.tags.remove(draft_id);
            }
        }
    }

    /// How many saved drafts point at each subtopic.
    pub fn tag_count(&self, path: &str) -> usize {
        self.tags
            .values()
            .filter(|tag| tag.as_str() == path)
            .count()
    }

    /// Dan Koe's five checks, in order, with whether each one is met.
    pub fn checklist(&self) -> Vec<(String, bool)> {
        let topics = self.topics.len();
        let subs_ok = topics > 0
            && self
                .topics
                .iter()
                .all(|topic| (3..=5).contains(&topic.subtopics.len()));
        let ideas_ok = subs_ok
            && self
                .topics
                .iter()
                .flat_map(|topic| topic.subtopics.iter())
                .all(|sub| (3..=5).contains(&sub.ideas.len()));
        vec![
            (
                "Point A: the life your reader wants to escape".into(),
                !self.point_a.trim().is_empty(),
            ),
            (
                "Point B: the life you help them move toward".into(),
                !self.point_b.trim().is_empty(),
            ),
            (
                format!("2-3 broad topics (now {topics})"),
                (2..=3).contains(&topics),
            ),
            ("3-5 subtopics under each topic".into(), subs_ok),
            ("3-5 post ideas under each subtopic".into(), ideas_ok),
        ]
    }

    pub fn idea_count(&self) -> usize {
        self.topics
            .iter()
            .flat_map(|topic| topic.subtopics.iter())
            .map(|sub| sub.ideas.len())
            .sum()
    }
}

pub fn map_status_line(map: &ContentMap) -> String {
    let subs: usize = map.topics.iter().map(|topic| topic.subtopics.len()).sum();
    let done = map.checklist().iter().filter(|(_, ok)| *ok).count();
    format!(
        "{} topics · {subs} subtopics · {} ideas · {done}/5 map steps done · {} tagged drafts",
        map.topics.len(),
        map.idea_count(),
        map.tags.len()
    )
}

/// A saved draft that starts from one map idea. Not posted anywhere.
pub fn map_draft(map: &ContentMap, topic: usize, sub: usize, idea: usize) -> Option<Draft> {
    let path = map.path(topic, sub)?;
    let text = map
        .topics
        .get(topic)?
        .subtopics
        .get(sub)?
        .ideas
        .get(idea)?
        .text
        .clone();
    Some(Draft {
        id: crate::drafts::new_id(),
        text,
        tone: Some(format!("Content Map · {path}")),
        source: "termy-xl-map".into(),
        created_at: crate::drafts::timestamp(),
        reply_to: None,
    })
}

pub fn content_map_file(data_root: &Path) -> PathBuf {
    data_root.join(CONTENT_MAP_FILE)
}

pub fn load_content_map(data_root: &Path) -> ContentMap {
    std::fs::read_to_string(content_map_file(data_root))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_content_map(data_root: &Path, map: &ContentMap) -> Result<(), String> {
    std::fs::create_dir_all(data_root).map_err(|error| error.to_string())?;
    let text = serde_json::to_string_pretty(map).map_err(|error| error.to_string())?;
    std::fs::write(content_map_file(data_root), text).map_err(|error| error.to_string())
}
