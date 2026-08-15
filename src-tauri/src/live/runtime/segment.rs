//! Encounter segment policy and packet-level combat routing.

use super::events::{
    DomainEnvelope, DomainEvent, DomainHit, EntityRef, EventMeta, HitKind, MonoTimeMs, SegmentId,
    SegmentReason, TimerKey, TimerScope,
};
use super::scheduler::{DeadlineScheduler, DueTimer, TimerTask};

pub const AUTOMATIC_BOUNDARY_DELAY_MS: u64 = 3_000;
pub const TRAINING_WINDOW_MS: u64 = 183_000;
/// Hard cap on one standard segment's lifetime, so field combat cannot keep
/// a single segment (and its history event chunks) growing for hours.
pub const MAX_SEGMENT_DURATION_MS: u64 = 20 * 60 * 1_000;
pub const TRAINING_DUMMY_MONSTER_IDS: [i32; 2] = [115, 122];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatGate {
    AllowAll,
    Only(EntityRef),
    BlockAll,
}

impl Default for CombatGate {
    fn default() -> Self {
        Self::AllowAll
    }
}

impl CombatGate {
    #[must_use]
    pub fn accepts(self, hit: &DomainHit) -> bool {
        match self {
            Self::AllowAll => true,
            Self::Only(target) => hit.target == target,
            Self::BlockAll => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryState {
    Clear,
    BoundaryDelay {
        reason: SegmentReason,
        deadline: MonoTimeMs,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleMode {
    Standard,
    TrainingArmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveSegment {
    pub id: SegmentId,
    pub started_at_mono_ms: MonoTimeMs,
    pub started_at_wall_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingMode {
    Standard {
        boundary: BoundaryState,
        training_armed: bool,
    },
    Training {
        target: EntityRef,
        monster_id: i32,
        deadline: MonoTimeMs,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentState {
    Idle {
        mode: IdleMode,
    },
    Recording {
        segment: ActiveSegment,
        mode: RecordingMode,
    },
    FrozenTraining {
        segment: ActiveSegment,
        target: EntityRef,
        monster_id: i32,
        ended_at_mono_ms: MonoTimeMs,
        ended_at_wall_ms: i64,
    },
}

impl Default for SegmentState {
    fn default() -> Self {
        Self::Idle {
            mode: IdleMode::Standard,
        }
    }
}

#[derive(Debug, Default)]
pub struct SegmentBatchDecision {
    pub segment_id: Option<SegmentId>,
    pub combat_gate: CombatGate,
    pub accept_hits_from_event_index: Option<u32>,
    pub transitions: Vec<DomainEvent>,
    /// Replayed only into the segment-scoped marker projection, not the full
    /// domain stream; the original global observations were already emitted.
    pub released_openers: Vec<DomainEnvelope>,
}

#[derive(Debug, Default)]
pub struct SegmentController {
    state: SegmentState,
    next_segment_id: u64,
    opener_buffer: Vec<DomainEnvelope>,
}

impl SegmentController {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn state(&self) -> &SegmentState {
        &self.state
    }

    #[must_use]
    pub fn current_segment_id(&self) -> Option<SegmentId> {
        match self.state {
            SegmentState::Idle { .. } => None,
            SegmentState::Recording { segment, .. }
            | SegmentState::FrozenTraining { segment, .. } => Some(segment.id),
        }
    }

    #[must_use]
    pub fn combat_gate(&self) -> CombatGate {
        match self.state {
            SegmentState::Idle { .. }
            | SegmentState::Recording {
                mode: RecordingMode::Standard { .. },
                ..
            } => CombatGate::AllowAll,
            SegmentState::Recording {
                mode: RecordingMode::Training { target, .. },
                ..
            } => CombatGate::Only(target),
            SegmentState::FrozenTraining { .. } => CombatGate::BlockAll,
        }
    }

    /// Examines the complete decoded packet before any projection runs. The
    /// returned segment id therefore applies to every event in the packet.
    pub fn preflight_batch(
        &mut self,
        events: &[DomainEnvelope],
        paused_at_batch_start: bool,
        scheduler: &mut DeadlineScheduler,
    ) -> SegmentBatchDecision {
        let Some(first) = events.first() else {
            return SegmentBatchDecision {
                segment_id: self.current_segment_id(),
                combat_gate: self.combat_gate(),
                ..SegmentBatchDecision::default()
            };
        };
        let meta = first.meta;
        let mut transitions = Vec::with_capacity(2);
        self.expire_elapsed_boundary(meta, scheduler, &mut transitions);

        let mut paused = paused_at_batch_start;
        let mut eligible_hit = None;
        let mut training_hit = None;
        for envelope in events {
            match &envelope.event {
                DomainEvent::PauseChanged { is_paused } => paused = *is_paused,
                DomainEvent::HitResolved(hit) if !paused => {
                    if eligible_hit.is_none() && is_eligible_player_damage(hit) {
                        eligible_hit = Some((hit, envelope.event_index));
                    }
                    if training_hit.is_none() && is_training_opener(hit) {
                        training_hit = Some((hit, envelope.event_index));
                    }
                }
                _ => {}
            }
        }

        let should_start_training = training_hit.is_some() && self.is_training_armed();
        let should_start_standard = matches!(self.state, SegmentState::Idle { .. })
            && eligible_hit.is_some()
            && !should_start_training;

        if should_start_training {
            let (hit, _) = training_hit.expect("checked above");
            self.end_recording(
                meta,
                SegmentReason::TrainingStarted,
                scheduler,
                &mut transitions,
            );
            self.start_training(meta, hit, scheduler, &mut transitions);
        } else if should_start_standard {
            let retain_training_arm = matches!(
                self.state,
                SegmentState::Idle {
                    mode: IdleMode::TrainingArmed
                }
            );
            self.start_standard(
                meta,
                SegmentReason::FirstCombat,
                scheduler,
                &mut transitions,
            );
            if retain_training_arm
                && let SegmentState::Recording {
                    mode: RecordingMode::Standard { training_armed, .. },
                    ..
                } = &mut self.state
            {
                *training_armed = true;
            }
        }

        let accept_hits_from_event_index = if should_start_training {
            training_hit.map(|(_, event_index)| event_index)
        } else if should_start_standard {
            eligible_hit.map(|(_, event_index)| event_index)
        } else {
            None
        };

        let segment_id = self.current_segment_id();
        let released_openers = if transitions
            .iter()
            .any(|event| matches!(event, DomainEvent::SegmentStarted { .. }))
        {
            self.drain_openers(segment_id.expect("segment just started"), meta.mono_ms())
        } else {
            Vec::new()
        };
        SegmentBatchDecision {
            segment_id,
            combat_gate: self.combat_gate(),
            accept_hits_from_event_index,
            transitions,
            released_openers,
        }
    }

    /// Arms an automatic boundary. Its deadline ends the current segment and
    /// enters idle even when no further packet arrives.
    pub fn arm_automatic_boundary(
        &mut self,
        meta: EventMeta,
        reason: SegmentReason,
        scheduler: &mut DeadlineScheduler,
    ) -> bool {
        let SegmentState::Recording {
            segment,
            mode: RecordingMode::Standard { boundary, .. },
        } = &mut self.state
        else {
            return false;
        };
        let deadline = meta.mono_ms().saturating_add(AUTOMATIC_BOUNDARY_DELAY_MS);
        if let BoundaryState::BoundaryDelay {
            deadline: current, ..
        } = boundary
            && *current <= deadline
        {
            return false;
        }
        *boundary = BoundaryState::BoundaryDelay { reason, deadline };
        scheduler.schedule(
            TimerKey::SegmentBoundary,
            TimerScope::Segment(segment.id),
            deadline,
            TimerTask::SegmentBoundary { reason },
        );
        true
    }

    pub fn on_timer(
        &mut self,
        due: DueTimer,
        fired_at_wall_ms: i64,
        scheduler: &mut DeadlineScheduler,
    ) -> Vec<DomainEvent> {
        match due.task {
            TimerTask::SegmentBoundary { reason } => {
                let SegmentState::Recording {
                    segment,
                    mode:
                        RecordingMode::Standard {
                            boundary:
                                BoundaryState::BoundaryDelay {
                                    reason: current_reason,
                                    deadline,
                                },
                            training_armed,
                        },
                } = self.state
                else {
                    return Vec::new();
                };
                if current_reason != reason || deadline != due.scheduled_for {
                    return Vec::new();
                }
                scheduler.invalidate_scope(TimerScope::Segment(segment.id));
                self.state = SegmentState::Idle {
                    mode: if training_armed {
                        IdleMode::TrainingArmed
                    } else {
                        IdleMode::Standard
                    },
                };
                vec![DomainEvent::SegmentEnded {
                    segment_id: segment.id,
                    reason,
                    ended_at_mono_ms: due.scheduled_for,
                    ended_at_wall_ms: fired_at_wall_ms,
                }]
            }
            TimerTask::SegmentMaxDuration { segment_id } => {
                let SegmentState::Recording {
                    segment,
                    mode: RecordingMode::Standard { training_armed, .. },
                } = self.state
                else {
                    return Vec::new();
                };
                let expected_deadline = segment
                    .started_at_mono_ms
                    .saturating_add(MAX_SEGMENT_DURATION_MS);
                if segment.id != segment_id || expected_deadline != due.scheduled_for {
                    return Vec::new();
                }
                scheduler.invalidate_scope(TimerScope::Segment(segment.id));
                self.state = SegmentState::Idle {
                    mode: if training_armed {
                        IdleMode::TrainingArmed
                    } else {
                        IdleMode::Standard
                    },
                };
                vec![DomainEvent::SegmentEnded {
                    segment_id: segment.id,
                    reason: SegmentReason::MaxDurationElapsed,
                    ended_at_mono_ms: due.scheduled_for,
                    ended_at_wall_ms: fired_at_wall_ms,
                }]
            }
            TimerTask::TrainingWindow { segment_id } => {
                let SegmentState::Recording {
                    segment,
                    mode:
                        RecordingMode::Training {
                            target,
                            monster_id,
                            deadline,
                        },
                } = self.state
                else {
                    return Vec::new();
                };
                if segment.id != segment_id || deadline != due.scheduled_for {
                    return Vec::new();
                }
                scheduler.invalidate_scope(TimerScope::Segment(segment.id));
                self.state = SegmentState::FrozenTraining {
                    segment,
                    target,
                    monster_id,
                    ended_at_mono_ms: due.scheduled_for,
                    ended_at_wall_ms: fired_at_wall_ms,
                };
                vec![DomainEvent::SegmentEnded {
                    segment_id: segment.id,
                    reason: SegmentReason::TrainingElapsed,
                    ended_at_mono_ms: due.scheduled_for,
                    ended_at_wall_ms: fired_at_wall_ms,
                }]
            }
            _ => Vec::new(),
        }
    }

    pub fn arm_training(&mut self, _scheduler: &mut DeadlineScheduler) {
        match &mut self.state {
            SegmentState::Idle { mode } => *mode = IdleMode::TrainingArmed,
            SegmentState::Recording {
                mode: RecordingMode::Standard { training_armed, .. },
                ..
            } => *training_armed = true,
            SegmentState::Recording {
                mode: RecordingMode::Training { .. },
                ..
            } => {}
            SegmentState::FrozenTraining { .. } => {
                self.state = SegmentState::Idle {
                    mode: IdleMode::TrainingArmed,
                };
            }
        }
    }

    pub fn manual_boundary(
        &mut self,
        meta: EventMeta,
        scheduler: &mut DeadlineScheduler,
    ) -> Vec<DomainEvent> {
        self.immediate_boundary(meta, SegmentReason::Manual, true, scheduler)
    }

    pub fn container_boundary(
        &mut self,
        meta: EventMeta,
        scheduler: &mut DeadlineScheduler,
    ) -> Vec<DomainEvent> {
        self.immediate_boundary(meta, SegmentReason::ContainerResync, false, scheduler)
    }

    pub fn shutdown_boundary(
        &mut self,
        meta: EventMeta,
        scheduler: &mut DeadlineScheduler,
    ) -> Vec<DomainEvent> {
        self.immediate_boundary(meta, SegmentReason::Shutdown, false, scheduler)
    }

    pub fn stop_training(
        &mut self,
        meta: EventMeta,
        scheduler: &mut DeadlineScheduler,
    ) -> Vec<DomainEvent> {
        self.immediate_boundary(meta, SegmentReason::Manual, false, scheduler)
    }

    /// Routes an already classified marker. Callers choose which skills are
    /// markers; this controller only owns their segment-boundary semantics.
    /// Clones only when the marker must be retained (idle buffer) or annotated
    /// for an active segment.
    pub fn route_marker(&mut self, marker: &DomainEnvelope) -> Option<DomainEnvelope> {
        match self.state {
            SegmentState::Idle { .. } => {
                let cutoff = marker.meta.mono_ms().saturating_sub(10_000);
                self.opener_buffer
                    .retain(|opener| opener.meta.mono_ms() >= cutoff);
                self.opener_buffer.push(marker.clone());
                None
            }
            SegmentState::Recording { segment, .. } => {
                let mut routed = marker.clone();
                routed.segment_id = Some(segment.id);
                Some(routed)
            }
            SegmentState::FrozenTraining { .. } => None,
        }
    }

    fn is_training_armed(&self) -> bool {
        matches!(
            self.state,
            SegmentState::Idle {
                mode: IdleMode::TrainingArmed
            } | SegmentState::Recording {
                mode: RecordingMode::Standard {
                    training_armed: true,
                    ..
                },
                ..
            }
        )
    }

    fn expire_elapsed_boundary(
        &mut self,
        meta: EventMeta,
        scheduler: &mut DeadlineScheduler,
        out: &mut Vec<DomainEvent>,
    ) {
        let SegmentState::Recording {
            segment,
            mode:
                RecordingMode::Standard {
                    boundary: BoundaryState::BoundaryDelay { reason, deadline },
                    training_armed,
                },
        } = self.state
        else {
            return;
        };
        // A packet captured exactly at the deadline belongs to the old
        // half-open grace window. The actor drains equal-time deadlines only
        // after it has processed the complete packet batch.
        if meta.mono_ms() <= deadline {
            return;
        }
        scheduler.invalidate_scope(TimerScope::Segment(segment.id));
        let delay_ms = meta.mono_ms().0.saturating_sub(deadline.0);
        let wall_delay_ms = i64::try_from(delay_ms).unwrap_or(i64::MAX);
        out.push(DomainEvent::SegmentEnded {
            segment_id: segment.id,
            reason,
            ended_at_mono_ms: deadline,
            ended_at_wall_ms: meta.captured_wall_ms.saturating_sub(wall_delay_ms),
        });
        self.state = SegmentState::Idle {
            mode: if training_armed {
                IdleMode::TrainingArmed
            } else {
                IdleMode::Standard
            },
        };
    }

    fn start_standard(
        &mut self,
        meta: EventMeta,
        reason: SegmentReason,
        scheduler: &mut DeadlineScheduler,
        out: &mut Vec<DomainEvent>,
    ) {
        let segment = self.next_segment(meta);
        self.state = SegmentState::Recording {
            segment,
            mode: RecordingMode::Standard {
                boundary: BoundaryState::Clear,
                training_armed: false,
            },
        };
        // Hard lifetime cap for standard segments. Training segments are
        // already bounded by TRAINING_WINDOW_MS and never schedule this.
        scheduler.schedule(
            TimerKey::SegmentMaxDuration {
                segment_id: segment.id,
            },
            TimerScope::Segment(segment.id),
            segment
                .started_at_mono_ms
                .saturating_add(MAX_SEGMENT_DURATION_MS),
            TimerTask::SegmentMaxDuration {
                segment_id: segment.id,
            },
        );
        out.push(started_event(segment, reason));
    }

    fn start_training(
        &mut self,
        meta: EventMeta,
        hit: &DomainHit,
        scheduler: &mut DeadlineScheduler,
        out: &mut Vec<DomainEvent>,
    ) {
        let segment = self.next_segment(meta);
        let deadline = meta.mono_ms().saturating_add(TRAINING_WINDOW_MS);
        let monster_id = hit
            .target_monster_id
            .expect("training opener has monster id");
        self.state = SegmentState::Recording {
            segment,
            mode: RecordingMode::Training {
                target: hit.target,
                monster_id,
                deadline,
            },
        };
        scheduler.schedule(
            TimerKey::TrainingWindow {
                segment_id: segment.id,
            },
            TimerScope::Segment(segment.id),
            deadline,
            TimerTask::TrainingWindow {
                segment_id: segment.id,
            },
        );
        out.push(started_event(segment, SegmentReason::TrainingStarted));
    }

    fn end_recording(
        &mut self,
        meta: EventMeta,
        reason: SegmentReason,
        scheduler: &mut DeadlineScheduler,
        out: &mut Vec<DomainEvent>,
    ) {
        let SegmentState::Recording { segment, .. } = self.state else {
            return;
        };
        scheduler.invalidate_scope(TimerScope::Segment(segment.id));
        out.push(ended_event(segment.id, meta, reason));
        self.state = SegmentState::Idle {
            mode: IdleMode::Standard,
        };
    }

    fn immediate_boundary(
        &mut self,
        meta: EventMeta,
        reason: SegmentReason,
        rearm_training: bool,
        scheduler: &mut DeadlineScheduler,
    ) -> Vec<DomainEvent> {
        let training_active = matches!(
            self.state,
            SegmentState::Recording {
                mode: RecordingMode::Training { .. },
                ..
            } | SegmentState::FrozenTraining { .. }
                | SegmentState::Idle {
                    mode: IdleMode::TrainingArmed
                }
                | SegmentState::Recording {
                    mode: RecordingMode::Standard {
                        training_armed: true,
                        ..
                    },
                    ..
                }
        );
        let mut out = Vec::with_capacity(1);
        self.end_recording(meta, reason, scheduler, &mut out);
        if let SegmentState::FrozenTraining { segment, .. } = self.state {
            scheduler.invalidate_scope(TimerScope::Segment(segment.id));
        }
        self.state = SegmentState::Idle {
            mode: if rearm_training && training_active {
                IdleMode::TrainingArmed
            } else {
                IdleMode::Standard
            },
        };
        scheduler.cancel(TimerKey::SegmentBoundary);
        out
    }

    fn next_segment(&mut self, meta: EventMeta) -> ActiveSegment {
        self.next_segment_id = self.next_segment_id.wrapping_add(1).max(1);
        ActiveSegment {
            id: SegmentId(self.next_segment_id),
            started_at_mono_ms: meta.mono_ms(),
            started_at_wall_ms: meta.captured_wall_ms,
        }
    }

    fn drain_openers(
        &mut self,
        segment_id: SegmentId,
        started_at_mono_ms: MonoTimeMs,
    ) -> Vec<DomainEnvelope> {
        let mut openers = std::mem::take(&mut self.opener_buffer);
        let cutoff = started_at_mono_ms.saturating_sub(10_000);
        openers.retain(|opener| opener.meta.mono_ms() >= cutoff);
        for opener in &mut openers {
            opener.segment_id = Some(segment_id);
        }
        openers
    }
}

fn is_eligible_player_damage(hit: &DomainHit) -> bool {
    hit.kind == HitKind::Damage && hit.source_is_player && hit.amount > 0
}

fn is_training_opener(hit: &DomainHit) -> bool {
    is_eligible_player_damage(hit)
        && hit.source_is_local_player
        && hit
            .target_monster_id
            .is_some_and(|id| TRAINING_DUMMY_MONSTER_IDS.contains(&id))
}

fn started_event(segment: ActiveSegment, reason: SegmentReason) -> DomainEvent {
    DomainEvent::SegmentStarted {
        segment_id: segment.id,
        reason,
        started_at_mono_ms: segment.started_at_mono_ms,
        started_at_wall_ms: segment.started_at_wall_ms,
    }
}

fn ended_event(segment_id: SegmentId, meta: EventMeta, reason: SegmentReason) -> DomainEvent {
    DomainEvent::SegmentEnded {
        segment_id,
        reason,
        ended_at_mono_ms: meta.mono_ms(),
        ended_at_wall_ms: meta.captured_wall_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::super::events::{BatchId, EntityKind, HitChannel, TimerKind};
    use super::*;

    fn meta(batch: u64, mono_ms: u64) -> EventMeta {
        EventMeta {
            batch_id: BatchId(batch),
            capture_sequence: batch,
            stream_id: 1,
            stream_epoch: 1,
            captured_wall_ms: 1_000_000 + mono_ms as i64,
            captured_mono_ns: mono_ms * 1_000_000,
            source_time_ms: None,
        }
    }

    fn hit_event(
        batch: u64,
        mono_ms: u64,
        target: EntityRef,
        monster_id: Option<i32>,
        local: bool,
    ) -> DomainEnvelope {
        let meta = meta(batch, mono_ms);
        DomainEnvelope {
            sequence: batch,
            batch_id: meta.batch_id,
            occurred_at_ms: meta.captured_wall_ms,
            meta,
            event_index: 0,
            segment_id: None,
            event: DomainEvent::HitResolved(DomainHit {
                channel: HitChannel::ToMe,
                source: Some(EntityRef {
                    uuid: super::super::events::EntityUuid(10),
                    generation: 1,
                }),
                packet_owner: None,
                resolved_owner: None,
                target,
                source_kind: Some(EntityKind::Character),
                target_kind: EntityKind::Monster,
                source_monster_id: None,
                target_monster_id: monster_id,
                target_is_boss: false,
                source_is_player: true,
                source_is_local_player: local,
                skill_key: 100,
                skill_id: Some(100),
                type_flags: 0,
                kind: HitKind::Damage,
                amount: 1,
                has_loss_breakdown: true,
                hp_loss: 1,
                shield_loss: 0,
                is_lucky_bonus_only: false,
                property: None,
                damage_mode: None,
                effective_amount: None,
            }),
        }
    }

    fn marker_event(batch: u64, mono_ms: u64) -> DomainEnvelope {
        let meta = meta(batch, mono_ms);
        DomainEnvelope {
            sequence: batch,
            batch_id: meta.batch_id,
            occurred_at_ms: meta.captured_wall_ms,
            meta,
            event_index: 0,
            segment_id: None,
            event: DomainEvent::DataQualityIssue(
                super::super::events::DataQualityIssue::DecodeIssue {
                    opcode: 1,
                    category: super::super::events::DecodeIssueCategory::Unsupported,
                },
            ),
        }
    }

    fn pause_event(batch: u64, mono_ms: u64, is_paused: bool) -> DomainEnvelope {
        let meta = meta(batch, mono_ms);
        DomainEnvelope {
            sequence: batch,
            batch_id: meta.batch_id,
            occurred_at_ms: meta.captured_wall_ms,
            meta,
            event_index: 0,
            segment_id: None,
            event: DomainEvent::PauseChanged { is_paused },
        }
    }

    fn target(uuid: i64) -> EntityRef {
        EntityRef {
            uuid: super::super::events::EntityUuid(uuid),
            generation: 1,
        }
    }

    #[test]
    fn automatic_boundary_waits_three_seconds_then_rolls_whole_damage_batch() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let target = target(20);
        let first = [hit_event(1, 1_000, target, None, false)];
        assert_eq!(
            controller
                .preflight_batch(&first, false, &mut scheduler)
                .segment_id,
            Some(SegmentId(1))
        );

        assert!(controller.arm_automatic_boundary(
            meta(2, 1_100),
            SegmentReason::AutomaticObjective,
            &mut scheduler,
        ));
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(4_100)));

        let grace = [hit_event(3, 4_099, target, None, false)];
        let decision = controller.preflight_batch(&grace, false, &mut scheduler);
        assert_eq!(decision.segment_id, Some(SegmentId(1)));
        assert!(decision.transitions.is_empty());

        let due = scheduler.drain_due(MonoTimeMs(4_100)).remove(0);
        let ended = controller.on_timer(due, 1_004_100, &mut scheduler);
        assert!(matches!(
            ended.as_slice(),
            [DomainEvent::SegmentEnded {
                segment_id: SegmentId(1),
                reason: SegmentReason::AutomaticObjective,
                ended_at_mono_ms: MonoTimeMs(4_100),
                ..
            }]
        ));
        assert!(matches!(controller.state(), SegmentState::Idle { .. }));
        assert!(controller.route_marker(&marker_event(4, 4_200)).is_none());
        let next = [
            marker_event(5, 4_500),
            hit_event(5, 4_500, target, None, false),
        ];
        let decision = controller.preflight_batch(&next, false, &mut scheduler);
        assert_eq!(decision.segment_id, Some(SegmentId(2)));
        assert_eq!(decision.transitions.len(), 1);
        assert!(matches!(
            decision.transitions.as_slice(),
            [DomainEvent::SegmentStarted {
                segment_id: SegmentId(2),
                ..
            }]
        ));
        assert_eq!(decision.released_openers.len(), 1);
    }

    #[test]
    fn training_deadline_freezes_without_another_packet() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        controller.arm_training(&mut scheduler);
        let dummy = target(22);
        let packet = [hit_event(1, 2_000, dummy, Some(115), true)];
        let decision = controller.preflight_batch(&packet, false, &mut scheduler);
        assert_eq!(decision.segment_id, Some(SegmentId(1)));
        assert_eq!(decision.combat_gate, CombatGate::Only(dummy));
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(185_000)));

        let due = scheduler.drain_due(MonoTimeMs(185_000)).remove(0);
        let events = controller.on_timer(due, 1_185_000, &mut scheduler);
        assert!(matches!(
            events.as_slice(),
            [DomainEvent::SegmentEnded {
                reason: SegmentReason::TrainingElapsed,
                ended_at_mono_ms: MonoTimeMs(185_000),
                ..
            }]
        ));
        assert_eq!(controller.combat_gate(), CombatGate::BlockAll);
        assert_eq!(controller.current_segment_id(), Some(SegmentId(1)));
        assert!(matches!(
            controller.state(),
            SegmentState::FrozenTraining { .. }
        ));
    }

    #[test]
    fn arming_training_preserves_existing_automatic_boundary() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let packet = [hit_event(1, 1_000, target(20), None, false)];
        controller.preflight_batch(&packet, false, &mut scheduler);
        assert!(controller.arm_automatic_boundary(
            meta(2, 1_100),
            SegmentReason::Wipe,
            &mut scheduler,
        ));
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(4_100)));

        controller.arm_training(&mut scheduler);
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(4_100)));
        let due = scheduler.drain_due(MonoTimeMs(4_100)).remove(0);
        let ended = controller.on_timer(due, 1_004_100, &mut scheduler);

        assert!(matches!(
            ended.as_slice(),
            [DomainEvent::SegmentEnded {
                reason: SegmentReason::Wipe,
                ..
            }]
        ));
        assert!(matches!(
            controller.state(),
            SegmentState::Idle {
                mode: IdleMode::TrainingArmed
            }
        ));
    }

    #[test]
    fn training_arm_does_not_block_a_later_automatic_boundary() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let packet = [hit_event(1, 1_000, target(20), None, false)];
        controller.preflight_batch(&packet, false, &mut scheduler);
        controller.arm_training(&mut scheduler);

        assert!(controller.arm_automatic_boundary(
            meta(2, 1_100),
            SegmentReason::AutomaticObjective,
            &mut scheduler,
        ));
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(4_100)));
    }

    #[test]
    fn new_training_segment_rejects_hits_before_the_local_opener_in_same_batch() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        controller.arm_training(&mut scheduler);
        let dummy = target(22);
        let mut teammate_hit = hit_event(1, 2_000, dummy, Some(115), false);
        teammate_hit.event_index = 0;
        let mut local_opener = hit_event(1, 2_000, dummy, Some(115), true);
        local_opener.event_index = 1;

        let decision =
            controller.preflight_batch(&[teammate_hit, local_opener], false, &mut scheduler);

        assert_eq!(decision.accept_hits_from_event_index, Some(1));
        assert_eq!(decision.combat_gate, CombatGate::Only(dummy));
    }

    #[test]
    fn opener_buffer_is_released_into_the_next_segment() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        assert!(controller.route_marker(&marker_event(1, 900)).is_none());

        let packet = [hit_event(2, 1_000, target(20), None, false)];
        let decision = controller.preflight_batch(&packet, false, &mut scheduler);
        assert_eq!(decision.released_openers.len(), 1);
        assert_eq!(decision.released_openers[0].segment_id, Some(SegmentId(1)));
    }

    #[test]
    fn paused_hits_do_not_start_a_segment_before_same_batch_resume() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let target = target(20);

        let hit_then_resume = [
            hit_event(1, 1_000, target, None, true),
            pause_event(1, 1_000, false),
        ];
        let decision = controller.preflight_batch(&hit_then_resume, true, &mut scheduler);
        assert_eq!(decision.segment_id, None);
        assert!(decision.transitions.is_empty());

        let resume_then_hit = [
            pause_event(2, 1_100, false),
            hit_event(2, 1_100, target, None, true),
        ];
        let decision = controller.preflight_batch(&resume_then_hit, true, &mut scheduler);
        assert_eq!(decision.segment_id, Some(SegmentId(1)));
        assert!(matches!(
            decision.transitions.as_slice(),
            [DomainEvent::SegmentStarted { .. }]
        ));
    }

    #[test]
    fn manual_and_container_boundaries_end_immediately() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let packet = [hit_event(1, 1_000, target(20), None, false)];
        controller.preflight_batch(&packet, false, &mut scheduler);
        let ended = controller.manual_boundary(meta(2, 1_100), &mut scheduler);
        assert_eq!(ended.len(), 1);
        assert!(matches!(controller.state(), SegmentState::Idle { .. }));

        controller.preflight_batch(&packet, false, &mut scheduler);
        let ended = controller.container_boundary(meta(3, 1_200), &mut scheduler);
        assert_eq!(ended.len(), 1);
        assert!(matches!(controller.state(), SegmentState::Idle { .. }));
    }

    #[test]
    fn max_duration_timer_ends_segment_and_next_hit_starts_new_one() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let target = target(20);
        let packet = [hit_event(1, 1_000, target, None, false)];
        let decision = controller.preflight_batch(&packet, false, &mut scheduler);
        assert_eq!(decision.segment_id, Some(SegmentId(1)));

        // The lifetime cap is the only timer a fresh standard segment has.
        let deadline = MonoTimeMs(1_000 + MAX_SEGMENT_DURATION_MS);
        assert_eq!(scheduler.len(), 1);
        assert_eq!(scheduler.next_deadline(), Some(deadline));

        let due = scheduler.drain_due(deadline).remove(0);
        let ended = controller.on_timer(due, 1_000_000 + deadline.0 as i64, &mut scheduler);
        assert!(matches!(
            ended.as_slice(),
            [DomainEvent::SegmentEnded {
                segment_id: SegmentId(1),
                reason: SegmentReason::MaxDurationElapsed,
                ended_at_mono_ms,
                ..
            }] if *ended_at_mono_ms == deadline
        ));
        assert!(matches!(controller.state(), SegmentState::Idle { .. }));

        // The next player-damage batch seamlessly opens a fresh segment with
        // its own lifetime cap.
        let next = [hit_event(2, deadline.0 + 10, target, None, false)];
        let decision = controller.preflight_batch(&next, false, &mut scheduler);
        assert_eq!(decision.segment_id, Some(SegmentId(2)));
        assert!(matches!(
            decision.transitions.as_slice(),
            [DomainEvent::SegmentStarted { .. }]
        ));
        assert_eq!(
            scheduler.next_deadline(),
            Some(MonoTimeMs(deadline.0 + 10 + MAX_SEGMENT_DURATION_MS))
        );
    }

    #[test]
    fn max_duration_timer_is_invalidated_when_segment_ends_early() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let packet = [hit_event(1, 1_000, target(20), None, false)];
        controller.preflight_batch(&packet, false, &mut scheduler);
        assert_eq!(scheduler.len(), 1);

        let ended = controller.manual_boundary(meta(2, 2_000), &mut scheduler);
        assert_eq!(ended.len(), 1);
        assert!(scheduler.is_empty());
        assert_eq!(scheduler.next_deadline(), None);
    }

    #[test]
    fn max_duration_timer_rejects_stale_segment_id() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        let target = target(20);
        let packet = [hit_event(1, 1_000, target, None, false)];
        controller.preflight_batch(&packet, false, &mut scheduler);
        controller.manual_boundary(meta(2, 2_000), &mut scheduler);

        let packet = [hit_event(3, 3_000, target, None, false)];
        controller.preflight_batch(&packet, false, &mut scheduler);
        assert_eq!(controller.current_segment_id(), Some(SegmentId(2)));

        // A stale firing for the ended segment must not end the new one.
        let stale = DueTimer {
            key: TimerKey::SegmentMaxDuration {
                segment_id: SegmentId(1),
            },
            kind: TimerKind::SegmentMaxDuration,
            scope: TimerScope::Segment(SegmentId(1)),
            scheduled_for: MonoTimeMs(1_000 + MAX_SEGMENT_DURATION_MS),
            generation: 0,
            task: TimerTask::SegmentMaxDuration {
                segment_id: SegmentId(1),
            },
        };
        let ended = controller.on_timer(stale, 2_000_000, &mut scheduler);
        assert!(ended.is_empty());
        assert_eq!(controller.current_segment_id(), Some(SegmentId(2)));
    }

    #[test]
    fn training_segment_has_no_max_duration_timer() {
        let mut controller = SegmentController::new();
        let mut scheduler = DeadlineScheduler::new();
        controller.arm_training(&mut scheduler);
        let dummy = target(22);
        let packet = [hit_event(1, 2_000, dummy, Some(115), true)];
        let decision = controller.preflight_batch(&packet, false, &mut scheduler);
        assert_eq!(decision.segment_id, Some(SegmentId(1)));

        // Training segments are bounded by the training window alone.
        assert_eq!(scheduler.len(), 1);
        assert_eq!(
            scheduler.next_deadline(),
            Some(MonoTimeMs(2_000 + TRAINING_WINDOW_MS))
        );
    }
}
