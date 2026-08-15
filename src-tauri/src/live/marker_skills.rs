//! Whitelist of player "key skills" that should be annotated on the DPS
//! timeline (burst openers, class-defining cooldowns, etc.).
//!
//! Matching is done against the skill id reported by `ATTR_SKILL_ID`
//! (attribute 100), not against a damage hit. The whitelist is shared with
//! the frontend (name + icon) via `meter-data/KeySkillMarkers.json`.

use std::collections::HashSet;
use std::sync::LazyLock;

const KEY_SKILL_MARKERS_JSON: &str = include_str!("../../meter-data/KeySkillMarkers.json");

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct KeySkillMarker {
    skill_id: i32,
}

pub static KEY_SKILL_IDS: LazyLock<HashSet<i32>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<KeySkillMarker>>(KEY_SKILL_MARKERS_JSON)
        .expect("key skill marker table must be valid JSON")
        .into_iter()
        .map(|marker| marker.skill_id)
        .collect()
});

#[must_use]
pub fn is_key_skill_marker(skill_id: i32) -> bool {
    KEY_SKILL_IDS.contains(&skill_id)
}
