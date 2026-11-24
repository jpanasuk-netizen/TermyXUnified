//! Dan Koe's audience loop inside Termy XL: a content map (step 1), a vault of
//! posts that beat their author's usual numbers (step 2) and a hook check for
//! drafts (steps 3-5). Everything here is local. Nothing posts.

mod content_map;
mod hook;
mod vault;

pub use content_map::{
    CONTENT_MAP_FILE, ContentMap, MapIdea, MapPick, MapSubtopic, MapTopic, content_map_file,
    load_content_map, map_draft, map_status_line, save_content_map,
};
pub use hook::{HOOK_READY_AT, HookCheck, HookItem, HookPart, hook_check};
pub use vault::{
    Baseline, VAULT_FILE, VAULT_MAX, VAULT_MIN_TARGET, VIRAL_MULTIPLE, VaultEntry, ViralFlag,
    ViralVault, engagement_score, load_vault, parse_ai_reasons, rule_reasons, save_vault,
    vault_file, vault_reasons_prompt, vault_status_line, viral_flag, viral_flags,
};

#[cfg(test)]
mod tests;
