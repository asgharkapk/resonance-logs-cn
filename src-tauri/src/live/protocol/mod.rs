pub mod attrs;
pub mod decoder;

/// Passive-skill skill ids 1101..=1106 are the in-game player markers;
/// the displayed marker number is `skill_id - MARKER_SKILL_ID_BASE`.
pub const MARKER_SKILL_ID_BASE: i32 = 1100;
