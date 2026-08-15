//! Indexed voice-rule projection using the shared deadline scheduler.

use std::collections::{BTreeSet, HashMap};

use crate::live::bootstrap_snapshot::MonitorRuntimeSnapshot;
use crate::live::counter::engine::{CounterEngine, CounterNamespace};
use crate::live::runtime::entity_context::EntityContext;
use crate::live::runtime::events::{
    BuffState, BuffTransition, DomainEnvelope, DomainEvent, MonoTimeMs, TimerKey, TimerScope,
};
use crate::live::runtime::scheduler::{DeadlineScheduler, TimerTask};
use crate::voice::models::{MonsterBuffSourceScope, VoiceCueIntent, VoiceRule, VoiceTrigger};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum VoiceBuffScope {
    LocalPlayer,
    MonsterAnySource,
    MonsterLocalPlayerSource,
}

impl VoiceBuffScope {
    const fn timer_subject(self) -> u64 {
        match self {
            Self::LocalPlayer => 0,
            Self::MonsterAnySource => 1,
            Self::MonsterLocalPlayerSource => 2,
        }
    }

    const fn from_timer_subject(subject: u64) -> Option<Self> {
        match subject {
            0 => Some(Self::LocalPlayer),
            1 => Some(Self::MonsterAnySource),
            2 => Some(Self::MonsterLocalPlayerSource),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BuffAggregate {
    expires_mono_ms: Option<MonoTimeMs>,
    fantasy_tier: Option<u8>,
}

#[derive(Debug, Default)]
struct BuffAggregateBuilder {
    latest_expiry: Option<MonoTimeMs>,
    has_unbounded_instance: bool,
    fantasy_tier: Option<u8>,
}

impl BuffAggregateBuilder {
    fn record(&mut self, buff: &BuffState, entities: &EntityContext) {
        // Legacy registry lookup at snapshot time. Multi-instance conflicts
        // used to be arbitrary-order last-wins; max() is the deterministic
        // tie-break across instances of the same base id.
        let tier = entities
            .resolve_fantasy_remodel_level(
                buff.source.map(|source| source.uuid),
                buff.source_config_id,
            )
            .and_then(|level| u8::try_from(level).ok());
        self.fantasy_tier = self.fantasy_tier.max(tier);
        if buff.duration_ms.is_none_or(|duration| duration == 0) || buff.expires_mono_ms.is_none() {
            self.has_unbounded_instance = true;
            self.latest_expiry = None;
            return;
        }
        if self.has_unbounded_instance {
            return;
        }
        if let Some(expires) = buff.expires_mono_ms {
            self.latest_expiry = Some(self.latest_expiry.map_or(expires, |old| old.max(expires)));
        }
    }

    fn finish(self) -> BuffAggregate {
        BuffAggregate {
            expires_mono_ms: (!self.has_unbounded_instance)
                .then_some(self.latest_expiry)
                .flatten(),
            fantasy_tier: self.fantasy_tier,
        }
    }
}

type BuffAggregates = HashMap<VoiceBuffScope, HashMap<i32, BuffAggregate>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BuffEdge {
    scope: VoiceBuffScope,
    base_id: i32,
    gained: bool,
    fantasy_tier: Option<u8>,
}

#[derive(Debug, Default)]
pub struct VoiceProjection {
    rules: Vec<VoiceRule>,
    buff_rules_by_id: HashMap<i32, Vec<usize>>,
    boss_rules_by_skill: HashMap<i32, Vec<usize>>,
    counter_rules: Vec<usize>,
    /// Rule handles for every `*Expiring` buff trigger, so expiry syncing
    /// touches only timer-bearing rules instead of scanning `rules`.
    buff_expiry_rules: Vec<usize>,
    buff_aggregates: BuffAggregates,
    boss_expiries: HashMap<TimerKey, MonoTimeMs>,
    rule_set: u64,
    last_fire_ms: HashMap<usize, i64>,
    counter_crossed: HashMap<(i32, i32), bool>,
    /// Cues matched during this batch, drained by [`Self::take_cues`].
    pending_cues: Vec<VoiceCueIntent>,
    /// Scratch buffer for rule handles, reused across edge evaluation so a
    /// buff-heavy batch does not allocate once per matched base id.
    matched_rules: Vec<usize>,
}

impl VoiceProjection {
    /// Removes the cues matched since the last drain. The caller owns the
    /// `VoiceService` handle; this projection stays free of Tauri.
    pub fn take_cues(&mut self) -> Vec<VoiceCueIntent> {
        std::mem::take(&mut self.pending_cues)
    }

    pub fn reset_runtime(&mut self, scheduler: &mut DeadlineScheduler) {
        if self.rule_set != 0 {
            scheduler.invalidate_scope(TimerScope::RuleSet(self.rule_set));
        }
        self.buff_aggregates.clear();
        self.boss_expiries.clear();
        self.last_fire_ms.clear();
        self.counter_crossed.clear();
    }

    pub fn apply_config(
        &mut self,
        config: &MonitorRuntimeSnapshot,
        entities: &EntityContext,
        now_mono: MonoTimeMs,
        scheduler: &mut DeadlineScheduler,
    ) {
        if self.rule_set != 0 {
            scheduler.invalidate_scope(TimerScope::RuleSet(self.rule_set));
        }
        self.boss_expiries.clear();
        self.rule_set = self.rule_set.wrapping_add(1).max(1);
        self.rules.clone_from(&config.voice.rules);
        self.rebuild_rule_indexes();
        self.last_fire_ms.clear();
        self.counter_crossed.clear();
        self.buff_aggregates = collect_buff_aggregates(entities);
        self.sync_buff_expiries(now_mono, scheduler);
    }

    /// Rebuilds the trigger lookup indexes from `rules`. Every path that
    /// replaces `rules` must call this, so the indexes can never drift out of
    /// sync with the rule list they point into.
    fn rebuild_rule_indexes(&mut self) {
        self.buff_rules_by_id.clear();
        self.boss_rules_by_skill.clear();
        self.counter_rules.clear();
        self.buff_expiry_rules.clear();
        for (index, rule) in self.rules.iter().enumerate() {
            if !rule.enabled {
                continue;
            }
            match &rule.trigger {
                VoiceTrigger::BuffGained { buff_id }
                | VoiceTrigger::BuffLost { buff_id }
                | VoiceTrigger::BuffExpiring { buff_id, .. }
                | VoiceTrigger::MonsterBuffGained { buff_id, .. }
                | VoiceTrigger::MonsterBuffLost { buff_id, .. }
                | VoiceTrigger::MonsterBuffExpiring { buff_id, .. } => {
                    self.buff_rules_by_id
                        .entry(*buff_id)
                        .or_default()
                        .push(index);
                    if buff_expiry_rule(&rule.trigger).is_some() {
                        self.buff_expiry_rules.push(index);
                    }
                }
                VoiceTrigger::BossDbm { base_skill_id }
                | VoiceTrigger::BossDbmExpiring { base_skill_id, .. } => self
                    .boss_rules_by_skill
                    .entry(*base_skill_id)
                    .or_default()
                    .push(index),
                VoiceTrigger::CounterThreshold { .. } | VoiceTrigger::CounterExpiring { .. } => {
                    self.counter_rules.push(index);
                }
            }
        }
    }

    pub fn apply(
        &mut self,
        envelope: &DomainEnvelope,
        entities: &EntityContext,
        scheduler: &mut DeadlineScheduler,
    ) {
        if self.rules.is_empty() {
            return;
        }
        match &envelope.event {
            DomainEvent::BuffChanged(buff) => {
                // Only the local player's and the current target's buffs feed
                // voice aggregates (see `collect_buff_aggregates`). Teammate
                // buff churn would otherwise rebuild and diff both scopes for
                // nothing on every raid-wide buff packet.
                if !buff.target_roles.is_local_player && !buff.target_roles.is_current_target {
                    return;
                }
                let emit_edges = buff.transition != BuffTransition::Baseline;
                self.refresh_buff_aggregates(
                    entities,
                    emit_edges,
                    envelope.occurred_at_ms,
                    envelope.meta.mono_ms(),
                    scheduler,
                );
            }
            DomainEvent::EntityDisappeared { .. } => {
                self.refresh_buff_aggregates(
                    entities,
                    true,
                    envelope.occurred_at_ms,
                    envelope.meta.mono_ms(),
                    scheduler,
                );
            }
            DomainEvent::EntityAppeared { .. }
            | DomainEvent::IdentityChanged { .. }
            | DomainEvent::LocalPlayerChanged { .. }
            | DomainEvent::AttackTargetChanged { .. }
            | DomainEvent::DeathOccurred { .. } => {
                self.replace_buff_aggregates(collect_buff_aggregates(entities), false);
                self.sync_buff_expiries(envelope.meta.mono_ms(), scheduler);
            }
            DomainEvent::SceneChanged { .. } => self.cancel_boss_expiries(scheduler),
            DomainEvent::BossMechanicStarted(mechanic) => {
                // The index map cannot stay borrowed across the `&mut self`
                // calls below, so stage the handles in the reusable scratch
                // buffer rather than allocating a fresh Vec per mechanic.
                let mut matched = std::mem::take(&mut self.matched_rules);
                matched.clear();
                if let Some(indices) = self.boss_rules_by_skill.get(&mechanic.base_skill_id) {
                    matched.extend_from_slice(indices);
                }
                for &index in &matched {
                    // Copy the trigger's payload out by value so `self.rules`
                    // is no longer borrowed when the handlers take `&mut self`.
                    let action = match self.rules.get(index).map(|rule| &rule.trigger) {
                        Some(VoiceTrigger::BossDbm { .. }) => Some(BossMechanicAction::Fire),
                        Some(VoiceTrigger::BossDbmExpiring { seconds_before, .. }) => {
                            Some(BossMechanicAction::Arm {
                                seconds_before: *seconds_before,
                            })
                        }
                        _ => None,
                    };
                    match action {
                        Some(BossMechanicAction::Fire) => {
                            self.fire_rule(index, envelope.occurred_at_ms, None);
                        }
                        Some(BossMechanicAction::Arm { seconds_before }) => {
                            self.sync_boss_expiry(
                                index,
                                mechanic.base_skill_id,
                                mechanic.expires_mono_ms,
                                seconds_before,
                                envelope.meta.mono_ms(),
                                scheduler,
                            );
                        }
                        None => {}
                    }
                }
                self.matched_rules = matched;
            }
            DomainEvent::DeadlineReached {
                key,
                scheduled_for,
                fired_at,
                ..
            } => self.on_deadline(*key, *scheduled_for, *fired_at, envelope.occurred_at_ms),
            _ => {}
        }
    }

    pub fn apply_counters(
        &mut self,
        counters: &CounterEngine,
        now_ms: i64,
        now_mono: MonoTimeMs,
        scheduler: &mut DeadlineScheduler,
    ) {
        if self.counter_rules.is_empty() {
            return;
        }
        // Counter rule handles are stable between `apply_config` calls, so the
        // index list is moved out and restored rather than cloned per flush.
        let counter_rules = std::mem::take(&mut self.counter_rules);
        for &index in &counter_rules {
            let Some(trigger) = self.rules.get(index).map(|rule| &rule.trigger) else {
                continue;
            };
            match *trigger {
                VoiceTrigger::CounterThreshold { rule_id, slot_id } => {
                    let crossed = counters
                        .slot_state(CounterNamespace::Normal, rule_id, slot_id)
                        .is_some_and(|slot| {
                            slot.effective_threshold
                                .is_some_and(|threshold| slot.current_count >= threshold)
                        });
                    let previous = self
                        .counter_crossed
                        .insert((rule_id, slot_id), crossed)
                        .unwrap_or(false);
                    if crossed && !previous {
                        self.fire_rule(index, now_ms, None);
                    }
                }
                VoiceTrigger::CounterExpiring {
                    rule_id,
                    slot_id,
                    seconds_before,
                } => {
                    let key = voice_key(
                        self.rule_set,
                        index,
                        0,
                        (u64::from(rule_id as u32) << 32) | u64::from(slot_id as u32),
                    );
                    scheduler.cancel(key);
                    if let Some(expiry_wall) = counters
                        .slot_state(CounterNamespace::Normal, rule_id, slot_id)
                        .and_then(|slot| slot.freeze_until_ms)
                    {
                        let remaining = expiry_wall.saturating_sub(now_ms).max(0) as u64;
                        schedule_voice(
                            key,
                            self.rule_set,
                            now_mono.saturating_add(remaining),
                            seconds_before,
                            now_mono,
                            scheduler,
                        );
                    }
                }
                _ => {}
            }
        }
        self.counter_rules = counter_rules;
    }

    /// Rebuilds the buff aggregates from `entities`, fires any resulting
    /// gained/lost edges, and re-arms expiry timers. Shared by the buff and
    /// entity-departure paths so both stay in lockstep.
    fn refresh_buff_aggregates(
        &mut self,
        entities: &EntityContext,
        emit_edges: bool,
        now_ms: i64,
        now_mono: MonoTimeMs,
        scheduler: &mut DeadlineScheduler,
    ) {
        let edges = self.replace_buff_aggregates(collect_buff_aggregates(entities), emit_edges);
        self.fire_buff_edges(&edges, now_ms);
        self.sync_buff_expiries(now_mono, scheduler);
    }

    fn replace_buff_aggregates(
        &mut self,
        current: BuffAggregates,
        emit_edges: bool,
    ) -> Vec<BuffEdge> {
        if !emit_edges {
            self.buff_aggregates = current;
            return Vec::new();
        }

        let mut keys = BTreeSet::new();
        for (scope, buffs) in &self.buff_aggregates {
            keys.extend(buffs.keys().map(|base_id| (*scope, *base_id)));
        }
        for (scope, buffs) in &current {
            keys.extend(buffs.keys().map(|base_id| (*scope, *base_id)));
        }

        let mut edges = Vec::new();
        for (scope, base_id) in keys {
            let previous = self
                .buff_aggregates
                .get(&scope)
                .and_then(|buffs| buffs.get(&base_id));
            let next = current.get(&scope).and_then(|buffs| buffs.get(&base_id));
            match (previous, next) {
                (None, Some(aggregate)) => edges.push(BuffEdge {
                    scope,
                    base_id,
                    gained: true,
                    fantasy_tier: aggregate.fantasy_tier,
                }),
                (Some(aggregate), None) => edges.push(BuffEdge {
                    scope,
                    base_id,
                    gained: false,
                    fantasy_tier: aggregate.fantasy_tier,
                }),
                _ => {}
            }
        }
        self.buff_aggregates = current;
        edges
    }

    fn fire_buff_edges(&mut self, edges: &[BuffEdge], now_ms: i64) {
        if edges.is_empty() {
            return;
        }
        // Handles are staged in the scratch buffer because firing needs
        // `&mut self` while `buff_rules_by_id` would still be borrowed.
        let mut matched = std::mem::take(&mut self.matched_rules);
        for edge in edges {
            matched.clear();
            let Some(indices) = self.buff_rules_by_id.get(&edge.base_id) else {
                continue;
            };
            matched.extend(
                indices
                    .iter()
                    .copied()
                    .filter(|index| matches_buff_edge(&self.rules[*index].trigger, *edge)),
            );
            for &index in &matched {
                self.fire_rule(index, now_ms, edge.fantasy_tier);
            }
        }
        self.matched_rules = matched;
    }

    fn sync_buff_expiries(&self, now: MonoTimeMs, scheduler: &mut DeadlineScheduler) {
        for &index in &self.buff_expiry_rules {
            let Some(rule) = self.rules.get(index) else {
                continue;
            };
            let Some((scope, base_id, seconds_before)) = buff_expiry_rule(&rule.trigger) else {
                continue;
            };
            let key = buff_voice_key(self.rule_set, index, scope, base_id);
            scheduler.cancel(key);
            let expiry = self
                .buff_aggregates
                .get(&scope)
                .and_then(|buffs| buffs.get(&base_id))
                .and_then(|aggregate| aggregate.expires_mono_ms);
            if let Some(expiry) = expiry {
                schedule_voice(key, self.rule_set, expiry, seconds_before, now, scheduler);
            }
        }
    }

    fn sync_boss_expiry(
        &mut self,
        index: usize,
        base_skill_id: i32,
        expires: MonoTimeMs,
        seconds_before: u32,
        now: MonoTimeMs,
        scheduler: &mut DeadlineScheduler,
    ) {
        let key = boss_voice_key(self.rule_set, index, base_skill_id);
        scheduler.cancel(key);
        self.boss_expiries.remove(&key);

        let deadline = expires.saturating_sub(u64::from(seconds_before).saturating_mul(1_000));
        if deadline < now {
            return;
        }
        scheduler.schedule(
            key,
            TimerScope::RuleSet(self.rule_set),
            deadline,
            TimerTask::VoiceExpiry,
        );
        self.boss_expiries.insert(key, deadline);
    }

    fn cancel_boss_expiries(&mut self, scheduler: &mut DeadlineScheduler) {
        for (key, _) in self.boss_expiries.drain() {
            scheduler.cancel(key);
        }
    }

    fn on_deadline(
        &mut self,
        key: TimerKey,
        scheduled_for: MonoTimeMs,
        fired_at: MonoTimeMs,
        now_ms: i64,
    ) {
        let TimerKey::VoiceExpiry {
            rule_set,
            rule_handle,
            subject,
            instance,
        } = key
        else {
            return;
        };
        if rule_set != self.rule_set {
            return;
        }
        let Ok(index) = usize::try_from(rule_handle) else {
            return;
        };
        let Some(rule) = self.rules.get(index) else {
            return;
        };

        if let Some((scope, base_id, seconds_before)) = buff_expiry_rule(&rule.trigger) {
            if VoiceBuffScope::from_timer_subject(subject) != Some(scope)
                || timer_instance_base_id(instance) != base_id
            {
                return;
            }
            let expected_deadline = self
                .buff_aggregates
                .get(&scope)
                .and_then(|buffs| buffs.get(&base_id))
                .and_then(|aggregate| aggregate.expires_mono_ms)
                .map(|expiry| {
                    expiry.saturating_sub(u64::from(seconds_before).saturating_mul(1_000))
                });
            if expected_deadline != Some(scheduled_for) || fired_at < scheduled_for {
                return;
            }
        } else if let VoiceTrigger::BossDbmExpiring { base_skill_id, .. } = &rule.trigger {
            if subject != BOSS_DBM_TIMER_SUBJECT
                || timer_instance_base_id(instance) != *base_skill_id
            {
                return;
            }
            let Some(expected_deadline) = self.boss_expiries.get(&key).copied() else {
                return;
            };
            if expected_deadline != scheduled_for || fired_at < scheduled_for {
                return;
            }
            self.boss_expiries.remove(&key);
        }

        self.fire_rule(index, now_ms, None);
    }

    fn fire_rule(&mut self, index: usize, now_ms: i64, tier: Option<u8>) {
        let Some(rule) = self.rules.get(index) else {
            return;
        };
        if !rule.enabled
            || self
                .last_fire_ms
                .get(&index)
                .is_some_and(|last| now_ms.saturating_sub(*last) < rule.cooldown_ms as i64)
        {
            return;
        }
        self.last_fire_ms.insert(index, now_ms);
        let phrase_id = tier
            .and_then(|tier| {
                rule.phrase_id_by_tier
                    .as_ref()
                    .and_then(|phrases| phrases.get(&tier))
            })
            .cloned()
            .unwrap_or_else(|| rule.phrase_id.clone());
        self.pending_cues.push(VoiceCueIntent {
            rule_id: rule.id.clone(),
            phrase_id,
            priority: rule.priority,
            triggered_at_ms: now_ms,
        });
    }
}

/// What a matched `BossMechanicStarted` rule should do, extracted by value so
/// the rule list is no longer borrowed when `&mut self` handlers run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BossMechanicAction {
    Fire,
    Arm { seconds_before: u32 },
}

fn collect_buff_aggregates(entities: &EntityContext) -> BuffAggregates {
    let mut builders: HashMap<VoiceBuffScope, HashMap<i32, BuffAggregateBuilder>> = HashMap::new();
    if let Some(local) = entities.local_player() {
        for buff in entities.active_buffs(local.uuid) {
            record_aggregate(&mut builders, VoiceBuffScope::LocalPlayer, buff, entities);
        }
    }

    if let Some(target) = entities.current_attack_target() {
        let local = entities.local_player();
        for buff in entities.active_buffs(target.uuid) {
            record_aggregate(
                &mut builders,
                VoiceBuffScope::MonsterAnySource,
                buff,
                entities,
            );
            // Legacy parity: "local player source" means the buff's raw source
            // is the player, not a summon/fantasy owned by the player.
            if local.is_some() && buff.source == local {
                record_aggregate(
                    &mut builders,
                    VoiceBuffScope::MonsterLocalPlayerSource,
                    buff,
                    entities,
                );
            }
        }
    }

    builders
        .into_iter()
        .map(|(scope, buffs)| {
            (
                scope,
                buffs
                    .into_iter()
                    .map(|(base_id, aggregate)| (base_id, aggregate.finish()))
                    .collect(),
            )
        })
        .collect()
}

fn record_aggregate(
    builders: &mut HashMap<VoiceBuffScope, HashMap<i32, BuffAggregateBuilder>>,
    scope: VoiceBuffScope,
    buff: &BuffState,
    entities: &EntityContext,
) {
    builders
        .entry(scope)
        .or_default()
        .entry(buff.base_id)
        .or_default()
        .record(buff, entities);
}

fn matches_buff_edge(trigger: &VoiceTrigger, edge: BuffEdge) -> bool {
    match trigger {
        VoiceTrigger::BuffGained { buff_id } => {
            edge.scope == VoiceBuffScope::LocalPlayer && edge.gained && *buff_id == edge.base_id
        }
        VoiceTrigger::BuffLost { buff_id } => {
            edge.scope == VoiceBuffScope::LocalPlayer && !edge.gained && *buff_id == edge.base_id
        }
        VoiceTrigger::MonsterBuffGained {
            buff_id,
            source_scope,
        } => edge.scope == monster_scope(*source_scope) && edge.gained && *buff_id == edge.base_id,
        VoiceTrigger::MonsterBuffLost {
            buff_id,
            source_scope,
        } => edge.scope == monster_scope(*source_scope) && !edge.gained && *buff_id == edge.base_id,
        _ => false,
    }
}

fn buff_expiry_rule(trigger: &VoiceTrigger) -> Option<(VoiceBuffScope, i32, u32)> {
    match trigger {
        VoiceTrigger::BuffExpiring {
            buff_id,
            seconds_before,
        } => Some((VoiceBuffScope::LocalPlayer, *buff_id, *seconds_before)),
        VoiceTrigger::MonsterBuffExpiring {
            buff_id,
            seconds_before,
            source_scope,
        } => Some((monster_scope(*source_scope), *buff_id, *seconds_before)),
        _ => None,
    }
}

const fn monster_scope(scope: MonsterBuffSourceScope) -> VoiceBuffScope {
    match scope {
        MonsterBuffSourceScope::AnySource => VoiceBuffScope::MonsterAnySource,
        MonsterBuffSourceScope::LocalPlayerSource => VoiceBuffScope::MonsterLocalPlayerSource,
    }
}

fn buff_voice_key(rule_set: u64, index: usize, scope: VoiceBuffScope, base_id: i32) -> TimerKey {
    voice_key(
        rule_set,
        index,
        scope.timer_subject(),
        u64::from(base_id as u32),
    )
}

const BOSS_DBM_TIMER_SUBJECT: u64 = 3;

fn boss_voice_key(rule_set: u64, index: usize, base_skill_id: i32) -> TimerKey {
    voice_key(
        rule_set,
        index,
        BOSS_DBM_TIMER_SUBJECT,
        u64::from(base_skill_id as u32),
    )
}

fn timer_instance_base_id(instance: u64) -> i32 {
    instance as u32 as i32
}

fn voice_key(rule_set: u64, index: usize, subject: u64, instance: u64) -> TimerKey {
    TimerKey::VoiceExpiry {
        rule_set,
        rule_handle: index as u64,
        subject,
        instance,
    }
}

fn schedule_voice(
    key: TimerKey,
    rule_set: u64,
    expires: MonoTimeMs,
    seconds_before: u32,
    now: MonoTimeMs,
    scheduler: &mut DeadlineScheduler,
) {
    let deadline = expires.saturating_sub(u64::from(seconds_before).saturating_mul(1_000));
    if deadline < now {
        return;
    }
    scheduler.schedule(
        key,
        TimerScope::RuleSet(rule_set),
        deadline,
        TimerTask::VoiceExpiry,
    );
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::live::runtime::events::{
        BatchId, EntityIdentityPatch, EntityKind, EntityRef, EntityUuid, EventMeta, FieldPatch,
        ObservedBuff, ObservedBuffChange, ProtocolBatch, ProtocolObservation,
    };

    fn buff(instance_id: i64, expires_mono_ms: Option<u64>) -> ObservedBuff {
        ObservedBuff {
            instance_id,
            base_id: 77,
            layer: 1,
            source_uuid: Some(EntityUuid(1)),
            source_config_id: None,
            duration_ms: expires_mono_ms.map(|_| 1_000),
            started_wall_ms: Some(1_000),
            expires_wall_ms: expires_mono_ms.map(|value| value as i64),
            started_mono_ms: Some(MonoTimeMs(1_000)),
            expires_mono_ms: expires_mono_ms.map(MonoTimeMs),
            effect_ids: Arc::from([]),
        }
    }

    fn batch(sequence: u64, observations: Vec<ProtocolObservation>) -> ProtocolBatch {
        ProtocolBatch {
            meta: EventMeta {
                batch_id: BatchId(sequence),
                capture_sequence: sequence,
                stream_id: 1,
                stream_epoch: 1,
                captured_wall_ms: sequence as i64 * 100,
                captured_mono_ns: sequence * 100_000_000,
                source_time_ms: None,
            },
            observations,
        }
    }

    fn local_context() -> EntityContext {
        let mut entities = EntityContext::new();
        entities.apply_batch(batch(
            1,
            vec![
                ProtocolObservation::EntityAppeared {
                    uuid: EntityUuid(1),
                    kind: EntityKind::Character,
                },
                ProtocolObservation::LocalPlayerChanged {
                    uuid: Some(EntityUuid(1)),
                },
            ],
        ));
        entities
    }

    fn buff_state(
        instance_id: i64,
        duration_ms: Option<u64>,
        expires_mono_ms: Option<u64>,
    ) -> BuffState {
        BuffState {
            target: EntityRef {
                uuid: EntityUuid(1),
                generation: 1,
            },
            instance_id,
            base_id: 77,
            layer: 1,
            source: None,
            resolved_owner: None,
            source_config_id: None,
            duration_ms,
            started_wall_ms: Some(1_000),
            expires_wall_ms: expires_mono_ms.map(|value| value as i64),
            started_mono_ms: Some(MonoTimeMs(1_000)),
            expires_mono_ms: expires_mono_ms.map(MonoTimeMs),
            effect_ids: Arc::from([]),
        }
    }

    #[test]
    fn multiple_instances_emit_only_first_gained_and_last_lost() {
        let mut projection = VoiceProjection::default();
        let mut entities = local_context();
        entities.apply_batch(batch(
            2,
            vec![ProtocolObservation::BuffChanged {
                target_uuid: EntityUuid(1),
                change: ObservedBuffChange::Applied {
                    buff: buff(10, Some(2_000)),
                },
            }],
        ));
        let gained = projection.replace_buff_aggregates(collect_buff_aggregates(&entities), true);
        assert_eq!(gained.len(), 1);
        assert!(gained[0].gained);

        entities.apply_batch(batch(
            3,
            vec![ProtocolObservation::BuffChanged {
                target_uuid: EntityUuid(1),
                change: ObservedBuffChange::Applied {
                    buff: buff(11, Some(3_000)),
                },
            }],
        ));
        assert!(
            projection
                .replace_buff_aggregates(collect_buff_aggregates(&entities), true)
                .is_empty()
        );

        entities.apply_batch(batch(
            4,
            vec![ProtocolObservation::BuffChanged {
                target_uuid: EntityUuid(1),
                change: ObservedBuffChange::Remove { instance_id: 10 },
            }],
        ));
        assert!(
            projection
                .replace_buff_aggregates(collect_buff_aggregates(&entities), true)
                .is_empty()
        );

        entities.apply_batch(batch(
            5,
            vec![ProtocolObservation::BuffChanged {
                target_uuid: EntityUuid(1),
                change: ObservedBuffChange::Remove { instance_id: 11 },
            }],
        ));
        let lost = projection.replace_buff_aggregates(collect_buff_aggregates(&entities), true);
        assert_eq!(lost.len(), 1);
        assert!(!lost[0].gained);
    }

    #[test]
    fn entity_disappearance_emits_one_lost_edge_for_each_aggregate() {
        let mut projection = VoiceProjection::default();
        let mut entities = local_context();
        entities.apply_batch(batch(
            2,
            vec![ProtocolObservation::BuffChanged {
                target_uuid: EntityUuid(1),
                change: ObservedBuffChange::Applied {
                    buff: buff(10, Some(2_000)),
                },
            }],
        ));
        projection.replace_buff_aggregates(collect_buff_aggregates(&entities), false);

        entities.apply_batch(batch(
            3,
            vec![ProtocolObservation::EntityDisappeared {
                uuid: EntityUuid(1),
            }],
        ));
        let lost = projection.replace_buff_aggregates(collect_buff_aggregates(&entities), true);

        assert_eq!(lost.len(), 1);
        assert_eq!(lost[0].scope, VoiceBuffScope::LocalPlayer);
        assert_eq!(lost[0].base_id, 77);
        assert!(!lost[0].gained);
    }

    #[test]
    fn permanent_instance_suppresses_aggregate_expiry() {
        let finite = buff_state(1, Some(1_000), Some(2_000));
        let permanent = buff_state(2, Some(0), None);

        let entities = EntityContext::new();
        let mut builder = BuffAggregateBuilder::default();
        builder.record(&finite, &entities);
        builder.record(&permanent, &entities);
        assert_eq!(builder.finish().expires_mono_ms, None);
    }

    #[test]
    fn local_source_scope_uses_original_source_not_summoner() {
        let mut entities = local_context();
        entities.apply_batch(batch(
            2,
            vec![
                ProtocolObservation::EntityAppeared {
                    uuid: EntityUuid(2),
                    kind: EntityKind::Monster,
                },
                ProtocolObservation::EntityAppeared {
                    uuid: EntityUuid(3),
                    kind: EntityKind::Monster,
                },
                ProtocolObservation::IdentityUpdated {
                    uuid: EntityUuid(3),
                    patch: EntityIdentityPatch {
                        owner_uuid: FieldPatch::Set(EntityUuid(1)),
                        ..Default::default()
                    },
                },
                ProtocolObservation::AttackTargetChanged {
                    actor_uuid: EntityUuid(1),
                    target_uuid: Some(EntityUuid(2)),
                },
                ProtocolObservation::BuffChanged {
                    target_uuid: EntityUuid(2),
                    change: ObservedBuffChange::Applied {
                        buff: ObservedBuff {
                            source_uuid: Some(EntityUuid(3)),
                            ..buff(10, Some(2_000))
                        },
                    },
                },
            ],
        ));

        // A summon owned by the local player is not the local player: raw
        // source matching excludes it from the local-player scope.
        let aggregates = collect_buff_aggregates(&entities);
        assert!(aggregates[&VoiceBuffScope::MonsterAnySource].contains_key(&77));
        assert!(
            !aggregates
                .get(&VoiceBuffScope::MonsterLocalPlayerSource)
                .is_some_and(|buffs| buffs.contains_key(&77))
        );

        entities.apply_batch(batch(
            3,
            vec![ProtocolObservation::BuffChanged {
                target_uuid: EntityUuid(2),
                change: ObservedBuffChange::Applied {
                    buff: ObservedBuff {
                        source_uuid: Some(EntityUuid(1)),
                        ..buff(11, Some(2_000))
                    },
                },
            }],
        ));
        let aggregates = collect_buff_aggregates(&entities);
        assert!(aggregates[&VoiceBuffScope::MonsterLocalPlayerSource].contains_key(&77));
    }

    #[test]
    fn aggregate_expiry_uses_latest_finite_instance() {
        let entities = EntityContext::new();
        let mut builder = BuffAggregateBuilder::default();
        builder.record(&buff_state(10, Some(1_000), Some(2_000)), &entities);
        builder.record(&buff_state(11, Some(2_000), Some(3_000)), &entities);
        assert_eq!(builder.finish().expires_mono_ms, Some(MonoTimeMs(3_000)));
    }

    #[test]
    fn permanent_aggregate_cancels_existing_expiry_timer() {
        let mut projection = VoiceProjection {
            rule_set: 1,
            rules: vec![VoiceRule {
                id: "buff-expiry".to_string(),
                enabled: true,
                trigger: VoiceTrigger::BuffExpiring {
                    buff_id: 77,
                    seconds_before: 1,
                },
                phrase_id: "buff-expiry".to_string(),
                priority: 1,
                cooldown_ms: 0,
                phrase_id_by_tier: None,
            }],
            buff_aggregates: HashMap::from([(
                VoiceBuffScope::LocalPlayer,
                HashMap::from([(
                    77,
                    BuffAggregate {
                        expires_mono_ms: Some(MonoTimeMs(5_000)),
                        fantasy_tier: None,
                    },
                )]),
            )]),
            ..Default::default()
        };
        projection.rebuild_rule_indexes();
        let mut scheduler = DeadlineScheduler::new();
        projection.sync_buff_expiries(MonoTimeMs(1_000), &mut scheduler);
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(4_000)));

        projection.buff_aggregates.insert(
            VoiceBuffScope::LocalPlayer,
            HashMap::from([(
                77,
                BuffAggregate {
                    expires_mono_ms: None,
                    fantasy_tier: None,
                },
            )]),
        );
        projection.sync_buff_expiries(MonoTimeMs(2_000), &mut scheduler);
        assert!(scheduler.is_empty());
    }

    #[test]
    fn expired_lead_time_is_not_scheduled() {
        let mut scheduler = DeadlineScheduler::new();
        schedule_voice(
            buff_voice_key(1, 0, VoiceBuffScope::LocalPlayer, 77),
            1,
            MonoTimeMs(2_000),
            1,
            MonoTimeMs(1_001),
            &mut scheduler,
        );
        assert!(scheduler.is_empty());
    }

    #[test]
    fn boss_expiry_refreshes_by_rule_and_base_skill() {
        let mut projection = VoiceProjection {
            rule_set: 1,
            ..Default::default()
        };
        let mut scheduler = DeadlineScheduler::new();
        let key = boss_voice_key(1, 2, 77);

        projection.sync_boss_expiry(
            2,
            77,
            MonoTimeMs(16_000),
            5,
            MonoTimeMs(1_000),
            &mut scheduler,
        );
        assert_eq!(
            projection.boss_expiries,
            HashMap::from([(key, MonoTimeMs(11_000))])
        );
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(11_000)));

        projection.sync_boss_expiry(
            2,
            77,
            MonoTimeMs(14_000),
            5,
            MonoTimeMs(4_000),
            &mut scheduler,
        );

        assert_eq!(
            projection.boss_expiries,
            HashMap::from([(key, MonoTimeMs(9_000))])
        );
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(9_000)));
    }

    #[test]
    fn scene_change_cancels_only_boss_expiries() {
        let mut projection = VoiceProjection {
            rule_set: 1,
            ..Default::default()
        };
        let mut scheduler = DeadlineScheduler::new();
        projection.sync_boss_expiry(
            2,
            77,
            MonoTimeMs(10_000),
            1,
            MonoTimeMs(1_000),
            &mut scheduler,
        );
        let buff_key = buff_voice_key(1, 3, VoiceBuffScope::LocalPlayer, 88);
        scheduler.schedule(
            buff_key,
            TimerScope::RuleSet(1),
            MonoTimeMs(4_000),
            TimerTask::VoiceExpiry,
        );

        projection.cancel_boss_expiries(&mut scheduler);

        assert!(projection.boss_expiries.is_empty());
        assert_eq!(scheduler.next_deadline(), Some(MonoTimeMs(4_000)));
    }

    // The cases below were migrated from the retired `voice::rules`
    // tracker. They now assert on `pending_cues` instead of returned intents.

    fn rule(id: &str, trigger: VoiceTrigger, phrase_id: &str, cooldown_ms: u64) -> VoiceRule {
        VoiceRule {
            id: id.to_string(),
            enabled: true,
            trigger,
            phrase_id: phrase_id.to_string(),
            priority: 1,
            cooldown_ms,
            phrase_id_by_tier: None,
        }
    }

    fn projection_with(rules: Vec<VoiceRule>) -> VoiceProjection {
        let mut projection = VoiceProjection {
            rule_set: 1,
            rules,
            ..Default::default()
        };
        projection.rebuild_rule_indexes();
        projection
    }

    fn fired_phrases(projection: &mut VoiceProjection) -> Vec<String> {
        projection
            .take_cues()
            .into_iter()
            .map(|cue| cue.phrase_id)
            .collect()
    }

    /// Applies `edges` and returns the phrases that fired.
    fn fire(projection: &mut VoiceProjection, now_ms: i64, edges: &[BuffEdge]) -> Vec<String> {
        projection.fire_buff_edges(edges, now_ms);
        fired_phrases(projection)
    }

    fn edge(scope: VoiceBuffScope, base_id: i32, gained: bool) -> BuffEdge {
        BuffEdge {
            scope,
            base_id,
            gained,
            fantasy_tier: None,
        }
    }

    #[test]
    fn local_and_monster_scopes_do_not_cross_talk() {
        let mut projection = projection_with(vec![
            rule("local", VoiceTrigger::BuffGained { buff_id: 42 }, "p1", 0),
            rule(
                "monster",
                VoiceTrigger::MonsterBuffGained {
                    buff_id: 42,
                    source_scope: MonsterBuffSourceScope::AnySource,
                },
                "p2",
                0,
            ),
        ]);

        // Same base id in both scopes: only the scope that saw the edge fires.
        assert_eq!(
            fire(
                &mut projection,
                1,
                &[edge(VoiceBuffScope::LocalPlayer, 42, true)]
            ),
            vec!["p1".to_string()]
        );
        assert_eq!(
            fire(
                &mut projection,
                2,
                &[edge(VoiceBuffScope::MonsterAnySource, 42, true)]
            ),
            vec!["p2".to_string()]
        );
    }

    #[test]
    fn monster_source_scopes_do_not_cross_talk() {
        let mut projection = projection_with(vec![
            rule(
                "any-gained",
                VoiceTrigger::MonsterBuffGained {
                    buff_id: 42,
                    source_scope: MonsterBuffSourceScope::AnySource,
                },
                "any-phrase",
                0,
            ),
            rule(
                "local-gained",
                VoiceTrigger::MonsterBuffGained {
                    buff_id: 42,
                    source_scope: MonsterBuffSourceScope::LocalPlayerSource,
                },
                "local-phrase",
                0,
            ),
        ]);

        // A buff applied by someone else only satisfies the any-source rule.
        assert_eq!(
            fire(
                &mut projection,
                1,
                &[edge(VoiceBuffScope::MonsterAnySource, 42, true)]
            ),
            vec!["any-phrase".to_string()]
        );
        assert_eq!(
            fire(
                &mut projection,
                2,
                &[edge(VoiceBuffScope::MonsterLocalPlayerSource, 42, true)]
            ),
            vec!["local-phrase".to_string()]
        );
    }

    #[test]
    fn gained_and_lost_triggers_only_match_their_own_direction() {
        let mut projection = projection_with(vec![
            rule("gained", VoiceTrigger::BuffGained { buff_id: 42 }, "on", 0),
            rule("lost", VoiceTrigger::BuffLost { buff_id: 42 }, "off", 0),
        ]);

        assert_eq!(
            fire(
                &mut projection,
                1,
                &[edge(VoiceBuffScope::LocalPlayer, 42, true)]
            ),
            vec!["on".to_string()]
        );
        assert_eq!(
            fire(
                &mut projection,
                2,
                &[edge(VoiceBuffScope::LocalPlayer, 42, false)]
            ),
            vec!["off".to_string()]
        );
    }

    #[test]
    fn respects_cooldown() {
        let mut projection = projection_with(vec![rule(
            "cooling",
            VoiceTrigger::BuffGained { buff_id: 42 },
            "p1",
            5_000,
        )]);
        let gained = [edge(VoiceBuffScope::LocalPlayer, 42, true)];

        assert_eq!(
            fire(&mut projection, 1_000, &gained),
            vec!["p1".to_string()]
        );
        // Within the cooldown window the same rule stays silent.
        assert!(fire(&mut projection, 3_000, &gained).is_empty());
        // Once the window elapses it fires again.
        assert_eq!(
            fire(&mut projection, 6_000, &gained),
            vec!["p1".to_string()]
        );
    }

    #[test]
    fn disabled_rule_never_fires() {
        let mut disabled = rule("off", VoiceTrigger::BuffGained { buff_id: 42 }, "p1", 0);
        disabled.enabled = false;
        let mut projection = projection_with(vec![disabled]);

        assert!(
            fire(
                &mut projection,
                1,
                &[edge(VoiceBuffScope::LocalPlayer, 42, true)]
            )
            .is_empty()
        );
    }

    #[test]
    fn buff_gained_selects_phrase_variant_by_fantasy_tier() {
        let mut tiered = rule(
            "tiered",
            VoiceTrigger::BuffGained { buff_id: 42 },
            "base",
            0,
        );
        tiered.phrase_id_by_tier = Some(HashMap::from([(3, "tier-3".to_string())]));
        let mut projection = projection_with(vec![tiered]);

        projection.fire_buff_edges(
            &[BuffEdge {
                scope: VoiceBuffScope::LocalPlayer,
                base_id: 42,
                gained: true,
                fantasy_tier: Some(3),
            }],
            1,
        );
        assert_eq!(fired_phrases(&mut projection), vec!["tier-3".to_string()]);
    }

    #[test]
    fn buff_gained_falls_back_to_base_phrase_for_unmapped_tier() {
        let mut tiered = rule(
            "tiered",
            VoiceTrigger::BuffGained { buff_id: 42 },
            "base",
            0,
        );
        tiered.phrase_id_by_tier = Some(HashMap::from([(3, "tier-3".to_string())]));
        let mut projection = projection_with(vec![tiered]);

        projection.fire_buff_edges(
            &[BuffEdge {
                scope: VoiceBuffScope::LocalPlayer,
                base_id: 42,
                gained: true,
                fantasy_tier: Some(5),
            }],
            1,
        );
        assert_eq!(fired_phrases(&mut projection), vec!["base".to_string()]);
    }

    #[test]
    fn baseline_buff_snapshot_rebaselines_without_firing() {
        let mut projection = projection_with(vec![rule(
            "gained",
            VoiceTrigger::BuffGained { buff_id: 77 },
            "p1",
            0,
        )]);
        let mut entities = local_context();
        entities.apply_batch(batch(
            2,
            vec![ProtocolObservation::BuffChanged {
                target_uuid: EntityUuid(1),
                change: ObservedBuffChange::Applied {
                    buff: buff(10, Some(2_000)),
                },
            }],
        ));
        let mut scheduler = DeadlineScheduler::new();

        // A container resync arrives as a Baseline transition: it must seed the
        // aggregate silently, exactly like the retired tracker's reset did.
        projection.refresh_buff_aggregates(&entities, false, 100, MonoTimeMs(100), &mut scheduler);
        assert!(projection.take_cues().is_empty());
        assert!(projection.buff_aggregates[&VoiceBuffScope::LocalPlayer].contains_key(&77));
    }

    #[test]
    fn counter_rules_stay_silent_when_no_slot_matches() {
        let mut projection = projection_with(vec![rule(
            "counter",
            VoiceTrigger::CounterThreshold {
                rule_id: 7,
                slot_id: 1,
            },
            "p1",
            0,
        )]);
        let mut scheduler = DeadlineScheduler::new();

        // No configured counter rule means no slot state, so the threshold
        // reads as uncrossed and the rule must not fire.
        projection.apply_counters(
            &CounterEngine::new(),
            1_000,
            MonoTimeMs(1_000),
            &mut scheduler,
        );

        assert!(projection.take_cues().is_empty());
        assert_eq!(projection.counter_crossed.get(&(7, 1)), Some(&false));
    }
}
