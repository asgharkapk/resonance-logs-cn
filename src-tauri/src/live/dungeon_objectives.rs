//! Dungeon objective reset rules.
//!
//! targets listed in `ResetIgnoreTargets.json`.

use std::collections::HashSet;
use std::sync::LazyLock;

/// Dungeon target IDs that should not trigger objective-based resets.
pub static RESET_IGNORE_TARGETS: LazyLock<HashSet<i32>> = LazyLock::new(|| {
    let data = include_str!("../../meter-data/ResetIgnoreTargets.json");
    serde_json::from_str::<Vec<i32>>(data)
        .unwrap_or_default()
        .into_iter()
        .collect()
});

pub fn classify_objective(
    target_id: i32,
    count: i32,
    complete: bool,
    active_target_id: &mut Option<i32>,
) -> bool {
    if !complete && count == 0 {
        *active_target_id = Some(target_id);
        if RESET_IGNORE_TARGETS.contains(&target_id) {
            log::info!(
                target: "app::live",
                "Reset suppressed: target_new_objective ignored target_id={target_id}"
            );
            return false;
        }
        log::info!(
            target: "app::live",
            "Reset rule matched: target_new_objective target_id={target_id}"
        );
        return true;
    }
    if complete && count > 0 {
        let effective_target_id = if target_id == 0 {
            active_target_id.unwrap_or(target_id)
        } else {
            target_id
        };
        if RESET_IGNORE_TARGETS.contains(&effective_target_id) {
            log::info!(
                target: "app::live",
                "Reset suppressed: target_completed ignored raw_target_id={target_id} effective_target_id={effective_target_id}"
            );
            return false;
        }
        log::info!(
            target: "app::live",
            "Reset rule matched: target_completed raw_target_id={target_id} effective_target_id={effective_target_id}"
        );
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ignored_target() -> i32 {
        *RESET_IGNORE_TARGETS
            .iter()
            .next()
            .expect("ResetIgnoreTargets.json is not empty")
    }

    #[test]
    fn new_objective_triggers_and_records_active_target() {
        let mut active = None;
        assert!(classify_objective(42, 0, false, &mut active));
        assert_eq!(active, Some(42));
    }

    #[test]
    fn new_ignored_objective_records_active_but_does_not_trigger() {
        let mut active = None;
        let ignored = ignored_target();
        assert!(!classify_objective(ignored, 0, false, &mut active));
        assert_eq!(active, Some(ignored));
    }

    #[test]
    fn completed_objective_triggers_only_with_progress() {
        let mut active = None;
        assert!(classify_objective(42, 1, true, &mut active));
        assert!(!classify_objective(42, 0, true, &mut active));
    }

    #[test]
    fn completed_ignored_objective_does_not_trigger() {
        let mut active = None;
        assert!(!classify_objective(ignored_target(), 3, true, &mut active));
    }

    #[test]
    fn completion_with_zero_target_falls_back_to_active_target() {
        let mut active = None;
        let ignored = ignored_target();
        // Offer an ignored objective, then complete it with target_id == 0:
        // the effective id is the ignored one, so no trigger.
        assert!(!classify_objective(ignored, 0, false, &mut active));
        assert!(!classify_objective(0, 1, true, &mut active));

        // Offer a normal objective, complete with target_id == 0: triggers.
        assert!(classify_objective(42, 0, false, &mut active));
        assert!(classify_objective(0, 1, true, &mut active));
    }

    #[test]
    fn progress_updates_do_not_trigger() {
        let mut active = None;
        assert!(!classify_objective(42, 1, false, &mut active));
        assert!(!classify_objective(42, 5, false, &mut active));
    }
}
