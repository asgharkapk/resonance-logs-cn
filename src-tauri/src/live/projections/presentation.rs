//! Replace-only live topic composition from incremental projection DTOs.

use crate::live::counter::engine::CounterSnapshot;
use crate::live::ipc::models::{
    DeathRecord, LiveBuffsPayload, LiveCombatPayload, LiveDataPayload, LiveDeathsPayload,
    LiveFantasyPayload, LiveMonsterPayload, LiveScenePayload, LiveStatusPayload,
    TrainingDummyPhase, TrainingDummyState,
};
use crate::live::projections::entity_monitor::EntityMonitorSnapshot;
use crate::live::runtime::events::SegmentId;
use crate::live::runtime::segment::{IdleMode, RecordingMode, SegmentState};

/// A live, in-progress combat payload paired with the segment it belongs to.
/// Callers derive this straight from [`CombatProjection`] (`segment_id()` /
/// `payload()`), so a live segment and its payload can never disagree about
/// which segment is active — unlike the old `Option<LiveDataPayload>` +
/// separately tracked id, which relied on an `expect()` to stay in sync.
///
/// [`CombatProjection`]: crate::live::projections::combat::projection::CombatProjection
#[derive(Debug, Clone)]
pub struct ActiveCombat {
    pub segment_id: SegmentId,
    pub payload: LiveDataPayload,
}

#[derive(Debug, Default)]
pub struct PresentationProjection {
    combat_revision: u64,
    status_revision: u64,
    buffs_revision: u64,
    monster_revision: u64,
    fantasy_revision: u64,
    deaths_revision: u64,
    scene_revision: u64,
    displayed_segment_id: Option<SegmentId>,
    displayed_combat: Option<LiveDataPayload>,
}

impl PresentationProjection {
    pub fn segment_started(&mut self, segment_id: SegmentId) {
        self.displayed_segment_id = Some(segment_id);
        self.displayed_combat = None;
    }

    /// Freezes only the combat (meter) payload. Counters are not segment
    /// scoped, so the status payload always reflects the live engine.
    pub fn freeze_segment(&mut self, segment_id: SegmentId, combat: LiveDataPayload) {
        if self.displayed_segment_id == Some(segment_id) {
            self.displayed_combat = Some(combat);
        }
    }

    pub fn clear_display(&mut self) {
        self.displayed_segment_id = None;
        self.displayed_combat = None;
    }

    /// Builds a combat payload and advances its revision (publication path).
    pub fn take_combat_payload(
        &mut self,
        active_combat: Option<ActiveCombat>,
        segment_state: &SegmentState,
    ) -> LiveCombatPayload {
        self.combat_revision = self.combat_revision.saturating_add(1);
        self.combat_payload(active_combat, segment_state)
    }

    /// Read-only combat payload for command-side bootstrap.
    #[must_use]
    pub fn peek_combat_payload(
        &self,
        active_combat: Option<ActiveCombat>,
        segment_state: &SegmentState,
    ) -> LiveCombatPayload {
        self.combat_payload(active_combat, segment_state)
    }

    pub fn take_status_payload(
        &mut self,
        monitored: &EntityMonitorSnapshot,
        counters: CounterSnapshot,
    ) -> LiveStatusPayload {
        self.status_revision = self.status_revision.saturating_add(1);
        self.status_payload(monitored, counters)
    }

    #[must_use]
    pub fn peek_status_payload(
        &self,
        monitored: &EntityMonitorSnapshot,
        counters: CounterSnapshot,
    ) -> LiveStatusPayload {
        self.status_payload(monitored, counters)
    }

    pub fn take_buffs_payload(&mut self, monitored: &EntityMonitorSnapshot) -> LiveBuffsPayload {
        self.buffs_revision = self.buffs_revision.saturating_add(1);
        self.buffs_payload(monitored)
    }

    #[must_use]
    pub fn peek_buffs_payload(&self, monitored: &EntityMonitorSnapshot) -> LiveBuffsPayload {
        self.buffs_payload(monitored)
    }

    pub fn take_monster_payload(
        &mut self,
        monitored: &EntityMonitorSnapshot,
    ) -> LiveMonsterPayload {
        self.monster_revision = self.monster_revision.saturating_add(1);
        self.monster_payload(monitored)
    }

    #[must_use]
    pub fn peek_monster_payload(&self, monitored: &EntityMonitorSnapshot) -> LiveMonsterPayload {
        self.monster_payload(monitored)
    }

    pub fn take_fantasy_payload(
        &mut self,
        monitored: &EntityMonitorSnapshot,
    ) -> LiveFantasyPayload {
        self.fantasy_revision = self.fantasy_revision.saturating_add(1);
        self.fantasy_payload(monitored)
    }

    #[must_use]
    pub fn peek_fantasy_payload(&self, monitored: &EntityMonitorSnapshot) -> LiveFantasyPayload {
        self.fantasy_payload(monitored)
    }

    /// Builds a deaths payload and advances its revision (publication path).
    pub fn take_deaths_payload(&mut self, deaths: Vec<DeathRecord>) -> LiveDeathsPayload {
        self.deaths_revision = self.deaths_revision.saturating_add(1);
        self.deaths_payload(deaths)
    }

    /// Read-only deaths payload for command-side bootstrap.
    #[must_use]
    pub fn peek_deaths_payload(&self, deaths: Vec<DeathRecord>) -> LiveDeathsPayload {
        self.deaths_payload(deaths)
    }

    /// Builds a scene payload and advances its revision (publication path).
    pub fn take_scene_payload(
        &mut self,
        scene_id: Option<i32>,
        dungeon_difficulty: Option<i32>,
    ) -> LiveScenePayload {
        self.scene_revision = self.scene_revision.saturating_add(1);
        self.scene_payload(scene_id, dungeon_difficulty)
    }

    /// Read-only scene payload for command-side bootstrap.
    #[must_use]
    pub fn peek_scene_payload(
        &self,
        scene_id: Option<i32>,
        dungeon_difficulty: Option<i32>,
    ) -> LiveScenePayload {
        self.scene_payload(scene_id, dungeon_difficulty)
    }

    fn combat_payload(
        &self,
        active_combat: Option<ActiveCombat>,
        segment_state: &SegmentState,
    ) -> LiveCombatPayload {
        let (active_segment_id, combat) = match active_combat {
            Some(active) => (Some(active.segment_id), Some(active.payload)),
            None => (None, self.displayed_combat.clone()),
        };
        LiveCombatPayload {
            revision: self.combat_revision,
            active_segment_id: active_segment_id.map(|segment| segment.0),
            displayed_segment_id: self.displayed_segment_id.map(|segment| segment.0),
            combat,
            training: TrainingDummyState {
                phase: training_phase(segment_state),
            },
        }
    }

    fn status_payload(
        &self,
        monitored: &EntityMonitorSnapshot,
        counters: CounterSnapshot,
    ) -> LiveStatusPayload {
        LiveStatusPayload {
            revision: self.status_revision,
            counters: counters.counters,
            factor_counters: counters.factor_counters,
            factor_source_item_ids: counters.factor_source_item_ids,
            factor_slot_item_ids: counters.factor_slot_item_ids,
            season_id: counters.season_id,
            season_active_template_ids: counters.season_active_template_ids,
            skill_cds: monitored.skill_cds.clone(),
            panel_attrs: monitored.panel_attrs.clone(),
            shield_current_hp: monitored.shield_current_hp,
            shield_max_hp: monitored.shield_max_hp,
            shield_entries: monitored.shield_entries.clone(),
            fight_resource: monitored.fight_resource.clone(),
        }
    }

    fn buffs_payload(&self, monitored: &EntityMonitorSnapshot) -> LiveBuffsPayload {
        LiveBuffsPayload {
            revision: self.buffs_revision,
            local_buffs: monitored.local_buffs.clone(),
        }
    }

    fn monster_payload(&self, monitored: &EntityMonitorSnapshot) -> LiveMonsterPayload {
        LiveMonsterPayload {
            revision: self.monster_revision,
            boss_buffs: monitored.boss_buffs.clone(),
            teammate_buffs: monitored.teammate_buffs.clone(),
            boss_mechanics: monitored.boss_mechanics.clone(),
            hate_lists: monitored.hate_lists.clone(),
            stun: monitored.stun.clone(),
            player_names: monitored.player_names.clone(),
            monster_ids: monitored.monster_ids.clone(),
        }
    }

    fn fantasy_payload(&self, monitored: &EntityMonitorSnapshot) -> LiveFantasyPayload {
        LiveFantasyPayload {
            revision: self.fantasy_revision,
            teammate_fantasies: monitored.teammate_fantasies.clone(),
        }
    }

    fn deaths_payload(&self, deaths: Vec<DeathRecord>) -> LiveDeathsPayload {
        LiveDeathsPayload {
            revision: self.deaths_revision,
            deaths,
        }
    }

    fn scene_payload(
        &self,
        scene_id: Option<i32>,
        dungeon_difficulty: Option<i32>,
    ) -> LiveScenePayload {
        LiveScenePayload {
            revision: self.scene_revision,
            scene_id,
            dungeon_difficulty,
        }
    }
}

fn training_phase(state: &SegmentState) -> TrainingDummyPhase {
    match state {
        SegmentState::Idle {
            mode: IdleMode::Standard,
        }
        | SegmentState::Recording {
            mode: RecordingMode::Standard { .. },
            ..
        } => TrainingDummyPhase::Idle,
        SegmentState::Idle {
            mode: IdleMode::TrainingArmed,
        } => TrainingDummyPhase::Armed,
        SegmentState::Recording {
            mode: RecordingMode::Training { .. },
            ..
        } => TrainingDummyPhase::Running,
        SegmentState::FrozenTraining { .. } => TrainingDummyPhase::Finished,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::projections::entity_monitor::EntityMonitorSnapshot;

    #[test]
    fn peek_does_not_advance_revision() {
        let mut presentation = PresentationProjection::default();
        let monitored = EntityMonitorSnapshot::default();
        let first = presentation.take_status_payload(&monitored, CounterSnapshot::default());
        assert_eq!(first.revision, 1);
        let peeked = presentation.peek_status_payload(&monitored, CounterSnapshot::default());
        assert_eq!(peeked.revision, 1);
        let second = presentation.take_status_payload(&monitored, CounterSnapshot::default());
        assert_eq!(second.revision, 2);
    }

    #[test]
    fn peek_deaths_does_not_advance_revision() {
        let mut presentation = PresentationProjection::default();
        let first = presentation.take_deaths_payload(Vec::new());
        assert_eq!(first.revision, 1);
        let peeked = presentation.peek_deaths_payload(Vec::new());
        assert_eq!(peeked.revision, 1);
        let second = presentation.take_deaths_payload(Vec::new());
        assert_eq!(second.revision, 2);
    }

    #[test]
    fn peek_scene_does_not_advance_revision() {
        let mut presentation = PresentationProjection::default();
        let first = presentation.take_scene_payload(Some(101), Some(2));
        assert_eq!(first.revision, 1);
        assert_eq!(first.scene_id, Some(101));
        let peeked = presentation.peek_scene_payload(Some(101), Some(2));
        assert_eq!(peeked.revision, 1);
        let second = presentation.take_scene_payload(Some(101), Some(2));
        assert_eq!(second.revision, 2);
    }

    fn idle_state() -> SegmentState {
        SegmentState::Idle {
            mode: IdleMode::Standard,
        }
    }

    fn payload_with_total_dmg(total_dmg: &str) -> LiveDataPayload {
        LiveDataPayload {
            total_dmg: total_dmg.to_string(),
            ..LiveDataPayload::default()
        }
    }

    /// A container resync ends the segment (freezing its combat payload) but
    /// never runs `CombatProjection::start_segment` again on its own, so
    /// there is no active combat to report until the next real segment
    /// starts. The frozen payload from the just-ended segment must stay
    /// visible in the meantime, exactly as it did before the resync.
    #[test]
    fn frozen_segment_stays_visible_without_an_active_one() {
        let mut presentation = PresentationProjection::default();
        presentation.segment_started(SegmentId(1));
        let frozen = payload_with_total_dmg("1234");
        presentation.freeze_segment(SegmentId(1), frozen.clone());

        let payload = presentation.take_combat_payload(None, &idle_state());

        assert_eq!(payload.active_segment_id, None);
        assert_eq!(payload.displayed_segment_id, Some(1));
        assert_eq!(
            payload.combat.map(|combat| combat.total_dmg),
            Some(frozen.total_dmg)
        );
    }

    /// Once a segment is actively recording again, its live payload must
    /// take priority over whatever was frozen from the previous one — the
    /// active/frozen distinction is derived entirely from the `Option`
    /// passed in by the caller, not from any state mirrored inside
    /// `PresentationProjection` itself.
    #[test]
    fn active_segment_payload_shadows_the_frozen_one() {
        let mut presentation = PresentationProjection::default();
        presentation.segment_started(SegmentId(1));
        presentation.freeze_segment(SegmentId(1), payload_with_total_dmg("1234"));

        let live = payload_with_total_dmg("5678");
        let active = ActiveCombat {
            segment_id: SegmentId(1),
            payload: live.clone(),
        };

        let payload = presentation.take_combat_payload(Some(active), &idle_state());

        assert_eq!(payload.active_segment_id, Some(1));
        assert_eq!(
            payload.combat.map(|combat| combat.total_dmg),
            Some(live.total_dmg)
        );
    }
}
