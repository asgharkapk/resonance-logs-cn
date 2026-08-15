//! Segment-scoped timeline marker projection.

use std::collections::HashSet;

use crate::database::history_codec::HistoryCastKind;
use crate::live::marker_skills::is_key_skill_marker;
use crate::live::runtime::entity_context::EntityContext;
use crate::live::runtime::events::{
    BatchId, DomainEnvelope, DomainEvent, EntityRef, EntityUuid, FantasyTransition, SkillPhase,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineMarker {
    pub caster: EntityRef,
    pub skill_id: i64,
    pub kind: HistoryCastKind,
}

#[derive(Debug, Default)]
pub struct TimelineProjection {
    current_batch: Option<BatchId>,
    seen_boss_markers: HashSet<(EntityUuid, i64)>,
}

impl TimelineProjection {
    pub fn reset_runtime(&mut self) {
        self.current_batch = None;
        self.seen_boss_markers.clear();
    }

    pub fn classify(
        &mut self,
        envelope: &DomainEnvelope,
        entities: &EntityContext,
    ) -> Option<TimelineMarker> {
        if self.current_batch != Some(envelope.batch_id) {
            self.current_batch = Some(envelope.batch_id);
            self.seen_boss_markers.clear();
        }
        let marker = classify_marker(envelope, entities)?;
        if marker.kind == HistoryCastKind::BossSkill
            && !self
                .seen_boss_markers
                .insert((marker.caster.uuid, marker.skill_id))
        {
            return None;
        }
        Some(marker)
    }
}

/// Classify only facts with a real timeline consumer. Unknown casters are not
/// searched globally: configured key skills can be classified directly, while
/// boss classification uses one UUID lookup in EntityContext.
pub fn classify_marker(
    envelope: &DomainEnvelope,
    entities: &EntityContext,
) -> Option<TimelineMarker> {
    match &envelope.event {
        DomainEvent::SkillLifecycleChanged {
            caster,
            skill_id,
            phase,
            ..
        } if matches!(phase, SkillPhase::CastStarted | SkillPhase::Observed) => {
            let key_skill = is_key_skill_marker(*skill_id);
            let boss_skill = entities
                .entity(caster.uuid)
                .is_some_and(|entity| entity.identity.is_boss);
            let kind = if boss_skill {
                HistoryCastKind::BossSkill
            } else if key_skill {
                HistoryCastKind::KeySkill
            } else {
                return None;
            };
            Some(TimelineMarker {
                caster: *caster,
                skill_id: i64::from(*skill_id),
                kind,
            })
        }
        DomainEvent::FantasyChanged {
            transition: FantasyTransition::Summoned,
            fantasy,
        } => Some(TimelineMarker {
            caster: fantasy.summoner,
            skill_id: i64::from(fantasy.resonance_skill_id?),
            kind: HistoryCastKind::Fantasy,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::runtime::events::{
        EntityIdentityPatch, EntityKind, EventMeta, FieldPatch, MonoTimeMs, ProtocolBatch,
        ProtocolObservation,
    };

    fn event_meta(batch_id: u64) -> EventMeta {
        EventMeta {
            batch_id: BatchId(batch_id),
            capture_sequence: batch_id,
            stream_id: 0,
            stream_epoch: 0,
            captured_wall_ms: 1_000,
            captured_mono_ns: MonoTimeMs(1_000).0 * 1_000_000,
            source_time_ms: None,
        }
    }

    fn boss_marker(batch_id: u64, sequence: u64, boss: EntityRef) -> DomainEnvelope {
        DomainEnvelope {
            sequence,
            batch_id: BatchId(batch_id),
            occurred_at_ms: 1_000,
            meta: event_meta(batch_id),
            event_index: 0,
            segment_id: None,
            event: DomainEvent::SkillLifecycleChanged {
                caster: boss,
                skill_id: 77,
                phase: SkillPhase::CastStarted,
                target: None,
            },
        }
    }

    #[test]
    fn duplicate_boss_skill_is_suppressed_only_within_the_same_batch() {
        let mut entities = EntityContext::new();
        entities.apply_batch(ProtocolBatch {
            meta: event_meta(0),
            observations: vec![
                ProtocolObservation::EntityAppeared {
                    uuid: EntityUuid(9),
                    kind: EntityKind::Monster,
                },
                ProtocolObservation::IdentityUpdated {
                    uuid: EntityUuid(9),
                    patch: EntityIdentityPatch {
                        is_boss: FieldPatch::Set(true),
                        ..Default::default()
                    },
                },
            ],
        });
        let boss = entities.entity_ref(EntityUuid(9)).expect("boss entity");
        let mut projection = TimelineProjection::default();

        assert!(
            projection
                .classify(&boss_marker(1, 1, boss), &entities)
                .is_some()
        );
        assert!(
            projection
                .classify(&boss_marker(1, 2, boss), &entities)
                .is_none()
        );
        assert!(
            projection
                .classify(&boss_marker(2, 3, boss), &entities)
                .is_some()
        );
    }
}
