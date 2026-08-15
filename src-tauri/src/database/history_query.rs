//! Pure historical projections and backend-neutral detail/range DTOs.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Range;

use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};

use crate::live::ipc::models::DeathRecord;
use crate::live::projections::combat::accumulator::{
    CombatAccumulator, CombatHitFact, CombatMetric, CombatSourceStats, CombatTargetStats,
    CombatantStats,
};
use crate::live::projections::combat::stats::class::{
    ClassSpec, get_class_id_from_spec, get_class_spec,
};
use crate::live::projections::combat::stats::{CombatStats, Skill, SkillTargetStats};

use super::commands::EncounterSummaryDto;
use super::event_journal::{
    EncounterHistoryDescriptor, EventJournalError, StoredHistoryChunk, StoredProjection,
    load_all_chunks, load_chunks_for_range, load_encounter_descriptor, load_projection,
};
use super::history_codec::{
    HistoryCastKind, HistoryChunkDocument, HistoryCodecError, HistoryEntityContext,
    HistoryEnvelope, HistoryEvent, HistoryMetric, decode_history_chunk,
};

const KNOWN_QUALITY_FLAGS: i32 = (1 << 3) - 1;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum HistoryQualityFlag {
    IncompleteSegment,
    MissingEntityContext,
    SaturatedAmount,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterTotalsData {
    pub damage: String,
    pub boss_damage: String,
    pub healing: String,
    pub effective_healing: String,
    pub damage_taken: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterStatsData {
    pub total: String,
    pub effective_total: String,
    pub hits: String,
    pub critical_hits: String,
    pub critical_total: String,
    pub lucky_hits: String,
    pub lucky_total: String,
    pub trigger_hits: String,
    pub blocked_hits: String,
    pub lucky_block_hits: String,
}

impl Default for EncounterStatsData {
    fn default() -> Self {
        Self {
            total: "0".to_string(),
            effective_total: "0".to_string(),
            hits: "0".to_string(),
            critical_hits: "0".to_string(),
            critical_total: "0".to_string(),
            lucky_hits: "0".to_string(),
            lucky_total: "0".to_string(),
            trigger_hits: "0".to_string(),
            blocked_hits: "0".to_string(),
            lucky_block_hits: "0".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSkillData {
    pub skill_id: String,
    pub metric: HistoryMetric,
    pub property: Option<i32>,
    pub damage_mode: Option<i32>,
    pub stats: EncounterStatsData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterTargetBreakdownData {
    pub target_entity_id: String,
    pub target_display_uid: i64,
    pub target_name: Option<String>,
    pub target_monster_id: Option<i32>,
    pub is_boss: bool,
    pub stats: EncounterStatsData,
    pub skills: Vec<EncounterSkillData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSourceBreakdownData {
    pub source_monster_id: Option<i32>,
    pub stats: EncounterStatsData,
    pub skills: Vec<EncounterSkillData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDeathData {
    pub offset_ms: u64,
    pub source_entity_id: Option<String>,
    pub skill_id: Option<String>,
    pub replay: Option<DeathRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterEntityData {
    pub entity_id: String,
    pub display_uid: i64,
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    /// Resolved spec display name; `None` for monsters / unknown specs.
    /// `Option` fields decode as `None` from projections stored before this
    /// field existed, and are backfilled from `class_spec` at query time.
    #[serde(default)]
    pub class_spec_name: Option<String>,
    pub ability_score: Option<i32>,
    pub season_strength: Option<i32>,
    pub monster_id: Option<i32>,
    pub totals: EncounterTotalsData,
    pub skills: Vec<EncounterSkillData>,
    pub damage_targets: Vec<EncounterTargetBreakdownData>,
    pub healing_targets: Vec<EncounterTargetBreakdownData>,
    pub taken_sources: Vec<EncounterSourceBreakdownData>,
    pub deaths: Vec<EncounterDeathData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterChartPointData {
    pub offset_ms: u64,
    pub damage: String,
    pub healing: String,
    pub damage_taken: String,
}

/// Sparse per-entity bucket series: one row per (entity, metric), holding only
/// the buckets with a non-zero total. Recomputed from raw chunks at query time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterChartSeriesData {
    pub entity_id: String,
    pub metric: HistoryMetric,
    pub offsets_ms: Vec<u64>,
    pub totals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterMarkerData {
    pub offset_ms: u64,
    pub sequence: u64,
    pub caster_entity_id: String,
    pub skill_id: String,
    pub kind: HistoryCastKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDetailData {
    pub encounter_id: i32,
    pub summary: EncounterSummaryDto,
    pub detail_available: bool,
    pub quality_flags: Vec<HistoryQualityFlag>,
    pub start_ms: u64,
    pub end_ms_exclusive: u64,
    pub bucket_ms: u64,
    pub totals: EncounterTotalsData,
    pub entities: Vec<EncounterEntityData>,
    pub chart_points: Vec<EncounterChartPointData>,
    /// Always recomputed from chunks on load; stored snapshots leave it empty.
    #[serde(default)]
    pub series: Vec<EncounterChartSeriesData>,
    pub markers: Vec<EncounterMarkerData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct EncounterRangeData {
    pub encounter_id: i32,
    pub quality_flags: Vec<HistoryQualityFlag>,
    pub start_ms: u64,
    pub end_ms_exclusive: u64,
    pub bucket_ms: u64,
    pub totals: EncounterTotalsData,
    pub entities: Vec<EncounterEntityData>,
    pub chart_points: Vec<EncounterChartPointData>,
    #[serde(default)]
    pub series: Vec<EncounterChartSeriesData>,
    pub markers: Vec<EncounterMarkerData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailProjectionSnapshot {
    pub last_sequence: u64,
    pub contexts: BTreeMap<i64, HistoryEntityContext>,
    pub detail: EncounterDetailData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedProjectionSnapshot {
    pub last_sequence: u64,
    pub quality_flags: i32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct EncounterDetailQuery {
    summary: EncounterSummaryDto,
    descriptor: EncounterHistoryDescriptor,
    projection: Option<StoredProjection>,
    chunks: Vec<StoredHistoryChunk>,
    timeline_end_ms_exclusive: u64,
}

#[derive(Debug)]
pub struct EncounterRangeQuery {
    encounter_id: i32,
    descriptor: EncounterHistoryDescriptor,
    projection: StoredProjection,
    chunks: Vec<StoredHistoryChunk>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
struct ChartProjection {
    damage: u128,
    healing: u128,
    taken: u128,
}

#[derive(Debug, thiserror::Error)]
pub enum HistoryProjectionError {
    #[error("range start {start_ms} is after end {end_ms}")]
    ReversedRange { start_ms: u64, end_ms: u64 },
    #[error("chart bucket width must be greater than zero")]
    ZeroBucketWidth,
    #[error("projection serialization failed: {0}")]
    Encode(String),
    #[error("projection decoding failed: {0}")]
    Decode(String),
}

/// Pure reducer shared by live finalization and historical range replay.
#[derive(Debug)]
pub struct HistoryProjectionReducer {
    range: Range<u64>,
    bucket_ms: u64,
    collect_dynamic_series: bool,
    accept_context_events: bool,
    last_sequence: u64,
    quality: BTreeSet<HistoryQualityFlag>,
    contexts: BTreeMap<i64, HistoryEntityContext>,
    combat: CombatAccumulator,
    deaths: BTreeMap<i64, Vec<EncounterDeathData>>,
    chart: BTreeMap<u64, ChartProjection>,
    /// Per-actor mirror of `chart`: actor entity id -> bucket offset -> totals.
    entity_chart: BTreeMap<i64, BTreeMap<u64, ChartProjection>>,
    markers: Vec<EncounterMarkerData>,
}

impl HistoryProjectionReducer {
    pub fn new(range: Range<u64>, bucket_ms: u64) -> Result<Self, HistoryProjectionError> {
        if range.start > range.end {
            return Err(HistoryProjectionError::ReversedRange {
                start_ms: range.start,
                end_ms: range.end,
            });
        }
        if bucket_ms == 0 {
            return Err(HistoryProjectionError::ZeroBucketWidth);
        }
        Ok(Self {
            range,
            bucket_ms,
            collect_dynamic_series: true,
            accept_context_events: true,
            last_sequence: 0,
            quality: BTreeSet::new(),
            contexts: BTreeMap::new(),
            combat: CombatAccumulator::default(),
            deaths: BTreeMap::new(),
            chart: BTreeMap::new(),
            entity_chart: BTreeMap::new(),
            markers: Vec::new(),
        })
    }

    #[must_use]
    pub fn without_dynamic_series(mut self) -> Self {
        self.collect_dynamic_series = false;
        self
    }

    #[must_use]
    fn with_seeded_contexts_only(mut self) -> Self {
        self.accept_context_events = false;
        self
    }

    pub fn mark_incomplete(&mut self) {
        self.quality.insert(HistoryQualityFlag::IncompleteSegment);
    }

    pub fn add_quality_flags(&mut self, flags: impl IntoIterator<Item = HistoryQualityFlag>) {
        self.quality.extend(flags);
    }

    pub fn seed_contexts(&mut self, contexts: impl IntoIterator<Item = HistoryEntityContext>) {
        for context in contexts {
            self.contexts.insert(context.entity_id, context);
        }
    }

    pub fn observe_sequence(&mut self, sequence: u64) {
        self.last_sequence = self.last_sequence.max(sequence);
    }

    pub fn apply_document(&mut self, document: &HistoryChunkDocument) {
        for envelope in document.envelopes() {
            self.apply(&envelope);
        }
    }

    pub fn apply(&mut self, envelope: &HistoryEnvelope) {
        self.last_sequence = self.last_sequence.max(envelope.sequence);
        if let HistoryEvent::EntityContext(context) = &envelope.event {
            if self.accept_context_events {
                self.contexts.insert(context.entity_id, context.clone());
            }
            return;
        }
        if envelope.offset_ms < self.range.start || envelope.offset_ms >= self.range.end {
            return;
        }

        match &envelope.event {
            HistoryEvent::Hit(hit) => {
                let fact = CombatHitFact::from(hit);
                if self.combat.apply(&fact) {
                    self.quality.insert(HistoryQualityFlag::SaturatedAmount);
                }
                if self.collect_dynamic_series {
                    let bucket_offset = self.range.start
                        + ((envelope.offset_ms - self.range.start) / self.bucket_ms)
                            * self.bucket_ms;
                    let bucket = self.chart.entry(bucket_offset).or_default();
                    add_chart_amount(bucket, fact.metric, fact.amount, &mut self.quality);
                    let entity_bucket = self
                        .entity_chart
                        .entry(fact.actor_entity_id)
                        .or_default()
                        .entry(bucket_offset)
                        .or_default();
                    add_chart_amount(
                        entity_bucket,
                        fact.metric,
                        fact.amount,
                        &mut self.quality,
                    );
                }
            }
            HistoryEvent::SkillCast(cast) if self.collect_dynamic_series => {
                self.markers.push(EncounterMarkerData {
                    offset_ms: envelope.offset_ms,
                    sequence: envelope.sequence,
                    caster_entity_id: cast.caster_entity_id.to_string(),
                    skill_id: cast.skill_id.to_string(),
                    kind: cast.kind,
                })
            }
            HistoryEvent::SkillCast(_) => {}
            HistoryEvent::Death(death) => {
                self.deaths
                    .entry(death.entity_id)
                    .or_default()
                    .push(EncounterDeathData {
                        offset_ms: envelope.offset_ms,
                        source_entity_id: death.source_entity_id.map(|id| id.to_string()),
                        skill_id: death.skill_id.map(|id| id.to_string()),
                        replay: death.replay.as_ref().map(DeathRecord::from),
                    });
            }
            HistoryEvent::EntityContext(_) => {}
        }
    }

    #[cfg(test)]
    pub fn finish_detail(
        mut self,
        encounter_id: i32,
        summary: EncounterSummaryDto,
    ) -> DetailProjectionSnapshot {
        let combat = std::mem::take(&mut self.combat);
        self.finish_detail_with_combat(encounter_id, summary, &combat)
    }

    /// Build a finalized projection from the accumulator owned by live combat.
    pub fn finish_detail_with_combat(
        mut self,
        encounter_id: i32,
        summary: EncounterSummaryDto,
        combat: &CombatAccumulator,
    ) -> DetailProjectionSnapshot {
        let mut detail = self.build_detail(encounter_id, summary, combat);
        detail.start_ms = 0;
        detail.end_ms_exclusive = 0;
        detail.bucket_ms = 0;
        detail.chart_points.clear();
        detail.series.clear();
        detail.markers.clear();
        DetailProjectionSnapshot {
            last_sequence: self.last_sequence,
            contexts: self.contexts,
            detail,
        }
    }

    fn finish_range(mut self, encounter_id: i32) -> EncounterRangeData {
        let combat = std::mem::take(&mut self.combat);
        let detail = self.build_detail(encounter_id, empty_summary(encounter_id), &combat);
        EncounterRangeData {
            encounter_id: detail.encounter_id,
            quality_flags: detail.quality_flags,
            start_ms: detail.start_ms,
            end_ms_exclusive: detail.end_ms_exclusive,
            bucket_ms: detail.bucket_ms,
            totals: detail.totals,
            entities: detail.entities,
            chart_points: detail.chart_points,
            series: detail.series,
            markers: detail.markers,
        }
    }

    fn build_detail(
        &mut self,
        encounter_id: i32,
        summary: EncounterSummaryDto,
        combat: &CombatAccumulator,
    ) -> EncounterDetailData {
        if combat.is_saturated() {
            self.quality.insert(HistoryQualityFlag::SaturatedAmount);
        }

        let mut entity_ids = combat.entities.keys().copied().collect::<BTreeSet<_>>();
        entity_ids.extend(self.deaths.keys().copied());
        for entity_id in &entity_ids {
            if !self.contexts.contains_key(entity_id) {
                self.quality
                    .insert(HistoryQualityFlag::MissingEntityContext);
            }
        }
        for stats in combat.entities.values() {
            for target_id in stats
                .damage_targets
                .keys()
                .chain(stats.healing_targets.keys())
            {
                if !self.contexts.contains_key(target_id) {
                    self.quality
                        .insert(HistoryQualityFlag::MissingEntityContext);
                }
            }
        }
        for marker in &self.markers {
            if marker
                .caster_entity_id
                .parse::<i64>()
                .ok()
                .is_none_or(|id| !self.contexts.contains_key(&id))
            {
                self.quality
                    .insert(HistoryQualityFlag::MissingEntityContext);
            }
        }

        let entities = entity_ids
            .into_iter()
            .map(|entity_id| {
                let context = self.contexts.get(&entity_id);
                let stats = combat.entities.get(&entity_id);
                let inferred_spec = stats
                    .map(|stats| stats.class_spec)
                    .filter(|spec| *spec != ClassSpec::Unknown);
                let context_spec = context.and_then(|value| value.class_spec);
                let spec_name = inferred_spec.or_else(|| {
                    context_spec
                        .map(ClassSpec::from_i32)
                        .filter(|spec| *spec != ClassSpec::Unknown)
                });
                let damage_targets = stats.map_or_else(Vec::new, |stats| {
                    history_targets(
                        &stats.damage_targets,
                        &stats.damage_skills,
                        HistoryMetric::Damage,
                        &self.contexts,
                    )
                });
                let healing_targets = stats.map_or_else(Vec::new, |stats| {
                    history_targets(
                        &stats.healing_targets,
                        &stats.healing_skills,
                        HistoryMetric::Healing,
                        &self.contexts,
                    )
                });
                let taken_sources =
                    stats.map_or_else(Vec::new, |stats| history_sources(&stats.taken_sources));
                let deaths = self.deaths.remove(&entity_id).unwrap_or_default();
                EncounterEntityData {
                    entity_id: entity_id.to_string(),
                    display_uid: context.map_or(entity_id, |value| value.display_uid),
                    name: context.and_then(|value| value.name.clone()),
                    class_id: inferred_spec
                        .map(get_class_id_from_spec)
                        .or_else(|| context.and_then(|value| value.class_id)),
                    class_spec: inferred_spec
                        .map(|spec| spec as i32)
                        .or(context_spec),
                    class_spec_name: spec_name.map(get_class_spec),
                    ability_score: context.and_then(|value| value.ability_score),
                    season_strength: context.and_then(|value| value.season_strength),
                    monster_id: context.and_then(|value| value.monster_id),
                    totals: EncounterTotalsData {
                        damage: stats.map_or(0, |stats| stats.damage.total).to_string(),
                        boss_damage: stats
                            .map_or(0, |stats| stats.damage_boss_only.total)
                            .to_string(),
                        healing: stats.map_or(0, |stats| stats.healing.total).to_string(),
                        effective_healing: stats
                            .map_or(0, |stats| stats.healing.effective_total)
                            .to_string(),
                        damage_taken: stats.map_or(0, |stats| stats.taken.total).to_string(),
                    },
                    skills: stats.map_or_else(Vec::new, history_skills),
                    damage_targets,
                    healing_targets,
                    taken_sources,
                    deaths,
                }
            })
            .collect();
        let chart_points = std::mem::take(&mut self.chart)
            .into_iter()
            .map(|(offset_ms, point)| EncounterChartPointData {
                offset_ms,
                damage: point.damage.to_string(),
                healing: point.healing.to_string(),
                damage_taken: point.taken.to_string(),
            })
            .collect();
        let series = chart_series_from_entity_chart(std::mem::take(&mut self.entity_chart));
        self.markers.sort_unstable_by(|left, right| {
            left.offset_ms
                .cmp(&right.offset_ms)
                .then_with(|| left.sequence.cmp(&right.sequence))
        });

        EncounterDetailData {
            encounter_id,
            summary,
            detail_available: true,
            quality_flags: self.quality.iter().copied().collect(),
            start_ms: self.range.start,
            end_ms_exclusive: self.range.end,
            bucket_ms: self.bucket_ms,
            totals: EncounterTotalsData {
                damage: combat.totals.damage.to_string(),
                boss_damage: combat.totals.boss_damage.to_string(),
                healing: combat.totals.healing.to_string(),
                effective_healing: combat.totals.effective_healing.to_string(),
                damage_taken: combat.totals.damage_taken.to_string(),
            },
            entities,
            chart_points,
            series,
            markers: std::mem::take(&mut self.markers),
        }
    }
}

fn history_skills(stats: &CombatantStats) -> Vec<EncounterSkillData> {
    let mut skills = Vec::with_capacity(
        stats.damage_skills.len() + stats.healing_skills.len() + stats.taken_skills.len(),
    );
    skills.extend(
        stats
            .damage_skills
            .iter()
            .map(|(skill_id, skill)| history_skill(*skill_id, HistoryMetric::Damage, skill)),
    );
    skills.extend(
        stats
            .healing_skills
            .iter()
            .map(|(skill_id, skill)| history_skill(*skill_id, HistoryMetric::Healing, skill)),
    );
    skills.extend(
        stats
            .taken_skills
            .iter()
            .map(|(skill_id, skill)| history_skill(*skill_id, HistoryMetric::DamageTaken, skill)),
    );
    sort_history_skills(&mut skills);
    skills
}

fn history_targets(
    targets: &HashMap<i64, CombatTargetStats>,
    overall_skills: &HashMap<i64, Skill>,
    metric: HistoryMetric,
    contexts: &BTreeMap<i64, HistoryEntityContext>,
) -> Vec<EncounterTargetBreakdownData> {
    let mut targets = targets.iter().collect::<Vec<_>>();
    targets.sort_unstable_by(|left, right| {
        right
            .1
            .stats
            .total_value
            .cmp(&left.1.stats.total_value)
            .then_with(|| left.0.cmp(right.0))
    });
    targets
        .into_iter()
        .map(|(target_id, target)| {
            let context = contexts.get(target_id);
            let mut skills = target
                .skills
                .iter()
                .map(|(skill_id, stats)| EncounterSkillData {
                    skill_id: skill_id.to_string(),
                    metric,
                    property: overall_skills
                        .get(skill_id)
                        .and_then(|skill| skill.property),
                    damage_mode: overall_skills
                        .get(skill_id)
                        .and_then(|skill| skill.damage_mode),
                    stats: target_stats_data(stats),
                })
                .collect::<Vec<_>>();
            sort_history_skills(&mut skills);
            EncounterTargetBreakdownData {
                target_entity_id: target_id.to_string(),
                target_display_uid: context.map_or_else(
                    || crate::live::entity_id::uid_from_uuid(*target_id),
                    |value| value.display_uid,
                ),
                target_name: context.and_then(|value| value.name.clone()),
                target_monster_id: target
                    .stats
                    .target_monster_id
                    .or_else(|| context.and_then(|value| value.monster_id)),
                is_boss: target.is_boss,
                stats: target_stats_data(&target.stats),
                skills,
            }
        })
        .collect()
}

fn history_sources(
    sources: &HashMap<Option<i32>, CombatSourceStats>,
) -> Vec<EncounterSourceBreakdownData> {
    let mut sources = sources.iter().collect::<Vec<_>>();
    sources.sort_unstable_by(|left, right| {
        right
            .1
            .stats
            .total
            .cmp(&left.1.stats.total)
            .then_with(|| left.0.cmp(right.0))
    });
    sources
        .into_iter()
        .map(|(source_monster_id, source)| {
            let mut skills = source
                .skills
                .iter()
                .map(|(skill_id, skill)| {
                    history_skill(*skill_id, HistoryMetric::DamageTaken, skill)
                })
                .collect::<Vec<_>>();
            sort_history_skills(&mut skills);
            EncounterSourceBreakdownData {
                source_monster_id: *source_monster_id,
                stats: combat_stats_data(&source.stats),
                skills,
            }
        })
        .collect()
}

fn sort_history_skills(skills: &mut [EncounterSkillData]) {
    skills.sort_unstable_by(|left, right| {
        right
            .stats
            .total
            .len()
            .cmp(&left.stats.total.len())
            .then_with(|| right.stats.total.cmp(&left.stats.total))
            .then_with(|| left.metric.cmp(&right.metric))
            .then_with(|| left.skill_id.cmp(&right.skill_id))
    });
}

fn history_skill(skill_id: i64, metric: HistoryMetric, skill: &Skill) -> EncounterSkillData {
    EncounterSkillData {
        skill_id: skill_id.to_string(),
        metric,
        property: skill.property,
        damage_mode: skill.damage_mode,
        stats: skill_stats_data(skill),
    }
}

fn skill_stats_data(skill: &Skill) -> EncounterStatsData {
    EncounterStatsData {
        total: skill.total_value.to_string(),
        effective_total: skill.effective_total_value.to_string(),
        hits: skill.hits.to_string(),
        critical_hits: skill.crit_hits.to_string(),
        critical_total: skill.crit_total_value.to_string(),
        lucky_hits: skill.lucky_hits.to_string(),
        lucky_total: skill.lucky_total_value.to_string(),
        trigger_hits: skill.trigger_hits.to_string(),
        blocked_hits: skill.block_hits.to_string(),
        lucky_block_hits: skill.lucky_block_hits.to_string(),
    }
}

fn combat_stats_data(stats: &CombatStats) -> EncounterStatsData {
    EncounterStatsData {
        total: stats.total.to_string(),
        effective_total: stats.effective_total.to_string(),
        hits: stats.hits.to_string(),
        critical_hits: stats.crit_hits.to_string(),
        critical_total: stats.crit_total.to_string(),
        lucky_hits: stats.lucky_hits.to_string(),
        lucky_total: stats.lucky_total.to_string(),
        trigger_hits: stats.trigger_hits.to_string(),
        blocked_hits: stats.block_hits.to_string(),
        lucky_block_hits: stats.lucky_block_hits.to_string(),
    }
}

fn target_stats_data(stats: &SkillTargetStats) -> EncounterStatsData {
    EncounterStatsData {
        total: stats.total_value.to_string(),
        effective_total: stats.effective_total_value.to_string(),
        hits: stats.hits.to_string(),
        critical_hits: stats.crit_hits.to_string(),
        critical_total: stats.crit_total.to_string(),
        lucky_hits: stats.lucky_hits.to_string(),
        lucky_total: stats.lucky_total.to_string(),
        trigger_hits: stats.trigger_hits.to_string(),
        blocked_hits: "0".to_string(),
        lucky_block_hits: "0".to_string(),
    }
}

pub fn encode_detail_projection(
    snapshot: &DetailProjectionSnapshot,
) -> Result<EncodedProjectionSnapshot, HistoryProjectionError> {
    let encoded = rmp_serde::to_vec_named(snapshot)
        .map_err(|error| HistoryProjectionError::Encode(error.to_string()))?;
    let data = zstd::encode_all(&encoded[..], 3)
        .map_err(|error| HistoryProjectionError::Encode(error.to_string()))?;
    Ok(EncodedProjectionSnapshot {
        last_sequence: snapshot.last_sequence,
        quality_flags: quality_flags_to_bits(&snapshot.detail.quality_flags),
        data,
    })
}

pub fn decode_detail_projection(
    data: &[u8],
) -> Result<DetailProjectionSnapshot, HistoryProjectionError> {
    let decoded = zstd::decode_all(data)
        .map_err(|error| HistoryProjectionError::Decode(error.to_string()))?;
    let snapshot: DetailProjectionSnapshot = rmp_serde::from_slice(&decoded)
        .map_err(|error| HistoryProjectionError::Decode(error.to_string()))?;
    Ok(snapshot)
}

/// Read only compressed query inputs while borrowing the actor-owned connection.
pub fn load_encounter_detail_query(
    conn: &mut SqliteConnection,
    summary: EncounterSummaryDto,
) -> Result<EncounterDetailQuery, HistoryQueryError> {
    let descriptor = load_encounter_descriptor(conn, summary.id)?;
    let projection = load_projection(conn, summary.id)?;
    let chunks = if projection.is_some() {
        load_all_chunks(conn, summary.id)?
    } else {
        Vec::new()
    };
    let timeline_end_ms_exclusive = chunks
        .iter()
        .map(|chunk| chunk.end_offset_ms_exclusive)
        .max()
        .unwrap_or_default()
        .max(encounter_duration_ms(&summary));
    Ok(EncounterDetailQuery {
        summary,
        descriptor,
        projection,
        chunks,
        timeline_end_ms_exclusive,
    })
}

/// Decode and project a detail query after releasing the SQLite actor.
pub fn project_encounter_detail(
    query: EncounterDetailQuery,
    target_points: u32,
) -> Result<EncounterDetailData, HistoryQueryError> {
    let EncounterDetailQuery {
        summary,
        descriptor,
        projection,
        chunks,
        timeline_end_ms_exclusive,
    } = query;
    let Some(stored) = projection else {
        return Ok(unavailable_detail(summary, descriptor.quality_flags));
    };
    let mut snapshot = decode_detail_projection(&stored.data)?;
    let quality_flags = validate_projection_metadata(&descriptor, &stored, &snapshot)?;
    let start_ms = 0;
    let end_ms_exclusive = timeline_end_ms_exclusive;
    let bucket_ms = bucket_width_for_points(start_ms, end_ms_exclusive, target_points)?;
    let chart = replay_chunks(
        summary.id,
        quality_flags,
        &snapshot,
        &chunks,
        start_ms,
        end_ms_exclusive,
        bucket_ms,
    )?;
    snapshot.detail.summary = summary;
    snapshot.detail.detail_available = true;
    snapshot.detail.start_ms = start_ms;
    snapshot.detail.end_ms_exclusive = end_ms_exclusive;
    snapshot.detail.bucket_ms = bucket_ms;
    snapshot.detail.chart_points = chart.chart_points;
    snapshot.detail.series = chart.series;
    snapshot.detail.markers = chart.markers;
    for entity in &mut snapshot.detail.entities {
        // Projections stored before `class_spec_name` existed decode it as
        // `None`; resolve it from the persisted spec discriminant instead.
        if entity.class_spec_name.is_none() {
            entity.class_spec_name = entity
                .class_spec
                .map(ClassSpec::from_i32)
                .filter(|spec| *spec != ClassSpec::Unknown)
                .map(get_class_spec);
        }
    }
    merge_quality_flags(
        &mut snapshot.detail.quality_flags,
        quality_flags_from_bits(quality_flags),
    );
    merge_quality_flags(&mut snapshot.detail.quality_flags, chart.quality_flags);
    Ok(snapshot.detail)
}

/// Replay only chunks intersecting the requested half-open range.
pub fn load_encounter_range_query(
    conn: &mut SqliteConnection,
    encounter_id: i32,
    start_ms: u64,
    end_ms_exclusive: u64,
) -> Result<EncounterRangeQuery, HistoryQueryError> {
    let descriptor = load_encounter_descriptor(conn, encounter_id)?;
    let projection = load_projection(conn, encounter_id)?
        .ok_or(HistoryQueryError::MissingProjection(encounter_id))?;
    let chunks = load_chunks_for_range(conn, encounter_id, start_ms, end_ms_exclusive)?;
    Ok(EncounterRangeQuery {
        encounter_id,
        descriptor,
        projection,
        chunks,
    })
}

/// Decode and replay a range after releasing the SQLite actor.
pub fn project_encounter_range(
    query: EncounterRangeQuery,
    start_ms: u64,
    end_ms_exclusive: u64,
) -> Result<EncounterRangeData, HistoryQueryError> {
    let snapshot = decode_detail_projection(&query.projection.data)?;
    let quality_flags =
        validate_projection_metadata(&query.descriptor, &query.projection, &snapshot)?;
    let bucket_ms = end_ms_exclusive.saturating_sub(start_ms).max(1);
    replay_chunks(
        query.encounter_id,
        quality_flags,
        &snapshot,
        &query.chunks,
        start_ms,
        end_ms_exclusive,
        bucket_ms,
    )
}

fn replay_chunks(
    encounter_id: i32,
    quality_flags: i32,
    snapshot: &DetailProjectionSnapshot,
    chunks: &[StoredHistoryChunk],
    start_ms: u64,
    end_ms_exclusive: u64,
    bucket_ms: u64,
) -> Result<EncounterRangeData, HistoryQueryError> {
    let mut reducer = HistoryProjectionReducer::new(start_ms..end_ms_exclusive, bucket_ms)?
        .with_seeded_contexts_only();
    reducer.seed_contexts(snapshot.contexts.values().cloned());
    reducer.add_quality_flags(quality_flags_from_bits(quality_flags));

    for chunk in chunks {
        let document = decode_history_chunk(&chunk.data, chunk.stream_kind)?;
        validate_chunk_metadata(chunk, &document)?;
        reducer.apply_document(&document);
    }
    Ok(reducer.finish_range(encounter_id))
}

fn validate_projection_metadata(
    descriptor: &EncounterHistoryDescriptor,
    stored: &StoredProjection,
    snapshot: &DetailProjectionSnapshot,
) -> Result<i32, HistoryQueryError> {
    if stored.encounter_id != descriptor.encounter_id
        || stored.last_sequence != snapshot.last_sequence
    {
        return Err(HistoryQueryError::ProjectionSequenceMismatch {
            encounter_id: descriptor.encounter_id,
        });
    }

    let snapshot_flags = quality_flags_to_bits(&snapshot.detail.quality_flags);
    if descriptor.quality_flags < 0
        || stored.quality_flags < 0
        || descriptor.quality_flags & !KNOWN_QUALITY_FLAGS != 0
        || stored.quality_flags & !KNOWN_QUALITY_FLAGS != 0
        || descriptor.quality_flags != stored.quality_flags
        || snapshot_flags & !stored.quality_flags != 0
    {
        return Err(HistoryQueryError::ProjectionQualityMismatch {
            encounter_id: descriptor.encounter_id,
        });
    }
    Ok(descriptor.quality_flags | stored.quality_flags | snapshot_flags)
}

fn validate_chunk_metadata(
    chunk: &StoredHistoryChunk,
    document: &HistoryChunkDocument,
) -> Result<(), HistoryQueryError> {
    let event_count = u64::try_from(document.len()).ok();
    let first_sequence = document.sequences.first().copied();
    let last_sequence = document.sequences.last().copied();
    let start_offset_ms = document.offsets_ms.iter().copied().min();
    let end_offset_ms_exclusive = document
        .offsets_ms
        .iter()
        .copied()
        .max()
        .and_then(|value| value.checked_add(1));
    if event_count != Some(chunk.event_count)
        || first_sequence != Some(chunk.first_sequence)
        || last_sequence != Some(chunk.last_sequence)
        || start_offset_ms != Some(chunk.start_offset_ms)
        || end_offset_ms_exclusive != Some(chunk.end_offset_ms_exclusive)
    {
        return Err(HistoryQueryError::ChunkMetadataMismatch {
            encounter_id: chunk.encounter_id,
        });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum HistoryQueryError {
    #[error(transparent)]
    Journal(#[from] EventJournalError),
    #[error(transparent)]
    Projection(#[from] HistoryProjectionError),
    #[error(transparent)]
    Chunk(#[from] HistoryCodecError),
    #[error("encounter {0} has no finalized projection")]
    MissingProjection(i32),
    #[error("encounter {encounter_id} projection sequence metadata is inconsistent")]
    ProjectionSequenceMismatch { encounter_id: i32 },
    #[error("encounter {encounter_id} projection quality metadata is inconsistent")]
    ProjectionQualityMismatch { encounter_id: i32 },
    #[error("encounter {encounter_id} chunk metadata is inconsistent")]
    ChunkMetadataMismatch { encounter_id: i32 },
    #[error("target points must be greater than zero")]
    ZeroTargetPoints,
}

fn encounter_duration_ms(summary: &EncounterSummaryDto) -> u64 {
    let duration_ms = if summary.duration.is_finite() && summary.duration > 0.0 {
        (summary.duration * 1_000.0).ceil().min(u64::MAX as f64) as u64
    } else {
        summary
            .ended_at_ms
            .and_then(|ended| ended.checked_sub(summary.started_at_ms))
            .and_then(|duration| u64::try_from(duration).ok())
            .unwrap_or_default()
    };
    duration_ms.max(1)
}

fn bucket_width_for_points(
    start_ms: u64,
    end_ms_exclusive: u64,
    target_points: u32,
) -> Result<u64, HistoryQueryError> {
    if target_points == 0 {
        return Err(HistoryQueryError::ZeroTargetPoints);
    }
    let duration = end_ms_exclusive.saturating_sub(start_ms).max(1);
    let points = u64::from(target_points);
    Ok((duration / points + u64::from(duration % points != 0)).max(1))
}

pub fn quality_flags_to_bits(flags: &[HistoryQualityFlag]) -> i32 {
    flags.iter().fold(0, |bits, flag| {
        bits | match flag {
            HistoryQualityFlag::IncompleteSegment => 1 << 0,
            HistoryQualityFlag::MissingEntityContext => 1 << 1,
            HistoryQualityFlag::SaturatedAmount => 1 << 2,
        }
    })
}

fn quality_flags_from_bits(bits: i32) -> Vec<HistoryQualityFlag> {
    let mut flags = Vec::with_capacity(3);
    if bits & (1 << 0) != 0 {
        flags.push(HistoryQualityFlag::IncompleteSegment);
    }
    if bits & (1 << 1) != 0 {
        flags.push(HistoryQualityFlag::MissingEntityContext);
    }
    if bits & (1 << 2) != 0 {
        flags.push(HistoryQualityFlag::SaturatedAmount);
    }
    flags
}

fn unavailable_detail(summary: EncounterSummaryDto, quality_flags: i32) -> EncounterDetailData {
    EncounterDetailData {
        encounter_id: summary.id,
        summary,
        detail_available: false,
        quality_flags: quality_flags_from_bits(quality_flags),
        start_ms: 0,
        end_ms_exclusive: 0,
        bucket_ms: 0,
        totals: EncounterTotalsData::default(),
        entities: Vec::new(),
        chart_points: Vec::new(),
        series: Vec::new(),
        markers: Vec::new(),
    }
}

fn merge_quality_flags(
    destination: &mut Vec<HistoryQualityFlag>,
    additional: impl IntoIterator<Item = HistoryQualityFlag>,
) {
    let mut flags = destination.iter().copied().collect::<BTreeSet<_>>();
    flags.extend(additional);
    *destination = flags.into_iter().collect();
}

fn add_chart_amount(
    bucket: &mut ChartProjection,
    metric: CombatMetric,
    amount: u128,
    quality: &mut BTreeSet<HistoryQualityFlag>,
) {
    match metric {
        CombatMetric::Damage => add_u128(&mut bucket.damage, amount, quality),
        CombatMetric::Healing => add_u128(&mut bucket.healing, amount, quality),
        CombatMetric::DamageTaken => add_u128(&mut bucket.taken, amount, quality),
    }
}

/// Drain per-actor buckets into sparse (entity, metric) series rows.
fn chart_series_from_entity_chart(
    entity_chart: BTreeMap<i64, BTreeMap<u64, ChartProjection>>,
) -> Vec<EncounterChartSeriesData> {
    const SELECTORS: [(HistoryMetric, fn(&ChartProjection) -> u128); 3] = [
        (HistoryMetric::Damage, |point| point.damage),
        (HistoryMetric::Healing, |point| point.healing),
        (HistoryMetric::DamageTaken, |point| point.taken),
    ];
    let mut series = Vec::new();
    for (entity_id, buckets) in entity_chart {
        let entity_id = entity_id.to_string();
        for (metric, select) in SELECTORS {
            let mut offsets_ms = Vec::new();
            let mut totals = Vec::new();
            for (offset, point) in &buckets {
                let value = select(point);
                if value > 0 {
                    offsets_ms.push(*offset);
                    totals.push(value.to_string());
                }
            }
            if !offsets_ms.is_empty() {
                series.push(EncounterChartSeriesData {
                    entity_id: entity_id.clone(),
                    metric,
                    offsets_ms,
                    totals,
                });
            }
        }
    }
    series
}

fn add_u128(target: &mut u128, value: u128, quality: &mut BTreeSet<HistoryQualityFlag>) {
    if let Some(total) = target.checked_add(value) {
        *target = total;
    } else {
        *target = u128::MAX;
        quality.insert(HistoryQualityFlag::SaturatedAmount);
    }
}

fn empty_summary(encounter_id: i32) -> EncounterSummaryDto {
    EncounterSummaryDto {
        id: encounter_id,
        started_at_ms: 0,
        ended_at_ms: None,
        total_dmg: "0".to_string(),
        total_heal: "0".to_string(),
        scene_id: None,
        dungeon_difficulty: None,
        duration: 0.0,
        active_combat_duration: None,
        local_player_id: None,
        bosses: Vec::new(),
        players: Vec::new(),
        remote_encounter_id: None,
        is_favorite: false,
        detail_available: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::history_codec::{
        HistoryDeath, HistoryHit, HistoryStream, encode_history_chunk,
    };
    use crate::live::projections::combat::accumulator::CombatHitFlags;
    use crate::live::projections::death::{
        DeathReplayBuff, DeathReplayDamage, DeathReplayParticipant, DeathReplaySnapshot,
    };

    fn hit(sequence: u64, offset_ms: u64, amount: u64) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms,
            event: HistoryEvent::Hit(HistoryHit {
                actor_entity_id: 1,
                source_entity_id: Some(1),
                target_entity_id: 2,
                skill_id: 7,
                base_skill_id: Some(7),
                metric: HistoryMetric::Damage,
                amount: amount.into(),
                effective_amount: amount.into(),
                has_loss_breakdown: true,
                hp_loss: amount.into(),
                shield_loss: 0,
                flags: 0,
                target_is_boss: false,
                target_monster_id: None,
                source_monster_id: None,
                property: None,
                damage_mode: None,
            }),
        }
    }

    fn death(
        sequence: u64,
        offset_ms: u64,
        replay: Option<DeathReplaySnapshot>,
    ) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms,
            event: HistoryEvent::Death(HistoryDeath {
                entity_id: 1,
                source_entity_id: Some(2),
                skill_id: Some(17_140_101),
                replay,
            }),
        }
    }

    fn death_replay(timestamp_ms: i64) -> DeathReplaySnapshot {
        DeathReplaySnapshot {
            victim_entity_uuid: 1,
            death_timestamp_ms: timestamp_ms,
            recent_damages: vec![DeathReplayDamage {
                timestamp_ms: timestamp_ms - 500,
                attacker_entity_uuid: Some(2),
                attacker_monster_type_id: Some(9_001),
                skill_key: 17_140_101,
                value: u128::MAX,
            }],
            victim_buffs: vec![DeathReplayBuff {
                base_id: 77,
                instance_id: 99,
                layer: 2,
                duration_ms: None,
                started_wall_ms: Some(timestamp_ms - 1_000),
                source_entity_uuid: Some(2),
                source_config_id: Some(700),
            }],
            participant_buffs: vec![DeathReplayParticipant {
                entity_uuid: Some(2),
                monster_type_id: Some(9_001),
                buffs: Vec::new(),
            }],
        }
    }

    fn metric_hit(sequence: u64, metric: HistoryMetric, amount: u64, flags: u8) -> HistoryEnvelope {
        metric_hit_at(sequence, sequence, metric, amount, flags)
    }

    fn metric_hit_at(
        sequence: u64,
        offset_ms: u64,
        metric: HistoryMetric,
        amount: u64,
        flags: u8,
    ) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms,
            event: HistoryEvent::Hit(HistoryHit {
                actor_entity_id: 1,
                source_entity_id: Some(if metric == HistoryMetric::DamageTaken {
                    2
                } else {
                    1
                }),
                target_entity_id: if metric == HistoryMetric::DamageTaken {
                    1
                } else {
                    2
                },
                skill_id: 7,
                base_skill_id: Some(7),
                metric,
                amount: amount.into(),
                effective_amount: if metric == HistoryMetric::Healing {
                    u128::from(amount / 2)
                } else {
                    0
                },
                has_loss_breakdown: metric == HistoryMetric::DamageTaken,
                hp_loss: amount.into(),
                shield_loss: 0,
                flags,
                target_is_boss: metric == HistoryMetric::Damage,
                target_monster_id: None,
                source_monster_id: None,
                property: None,
                damage_mode: None,
            }),
        }
    }

    fn stored_combat_chunk(
        encounter_id: i32,
        chunk_index: u64,
        events: Vec<HistoryEnvelope>,
    ) -> StoredHistoryChunk {
        let chunk = encode_history_chunk(encounter_id, HistoryStream::Combat, chunk_index, events)
            .expect("encode combat chunk");
        StoredHistoryChunk {
            encounter_id: chunk.encounter_id,
            stream_kind: chunk.stream_kind,
            chunk_index: chunk.chunk_index,
            first_sequence: chunk.first_sequence,
            last_sequence: chunk.last_sequence,
            start_offset_ms: chunk.start_offset_ms,
            end_offset_ms_exclusive: chunk.end_offset_ms_exclusive,
            event_count: chunk.event_count,
            data: chunk.data,
        }
    }

    fn stored_context_chunk(
        encounter_id: i32,
        chunk_index: u64,
        events: Vec<HistoryEnvelope>,
    ) -> StoredHistoryChunk {
        let chunk = encode_history_chunk(encounter_id, HistoryStream::Context, chunk_index, events)
            .expect("encode context chunk");
        StoredHistoryChunk {
            encounter_id: chunk.encounter_id,
            stream_kind: chunk.stream_kind,
            chunk_index: chunk.chunk_index,
            first_sequence: chunk.first_sequence,
            last_sequence: chunk.last_sequence,
            start_offset_ms: chunk.start_offset_ms,
            end_offset_ms_exclusive: chunk.end_offset_ms_exclusive,
            event_count: chunk.event_count,
            data: chunk.data,
        }
    }

    fn context_at(sequence: u64, offset_ms: u64, name: &str) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms,
            event: HistoryEvent::EntityContext(HistoryEntityContext {
                entity_id: 1,
                display_uid: 1,
                name: Some(name.to_string()),
                class_id: None,
                class_spec: None,
                ability_score: None,
                season_strength: None,
                monster_id: None,
            }),
        }
    }

    fn replay_range(range: Range<u64>, chunks: &[StoredHistoryChunk]) -> EncounterRangeData {
        let mut seed = HistoryProjectionReducer::new(0..0, 1).expect("seed reducer");
        seed.seed_contexts([HistoryEntityContext {
            entity_id: 1,
            display_uid: 1,
            name: Some("player".to_string()),
            class_id: None,
            class_spec: None,
            ability_score: None,
            season_strength: None,
            monster_id: None,
        }]);
        let mut snapshot = seed.finish_detail(1, empty_summary(1));
        snapshot.last_sequence = chunks
            .iter()
            .map(|chunk| chunk.last_sequence)
            .max()
            .unwrap_or_default();
        let intersecting = chunks
            .iter()
            .filter(|chunk| {
                chunk.start_offset_ms < range.end && chunk.end_offset_ms_exclusive > range.start
            })
            .cloned()
            .collect::<Vec<_>>();
        let bucket_ms = range.end.saturating_sub(range.start).max(1);
        replay_chunks(
            1,
            0,
            &snapshot,
            &intersecting,
            range.start,
            range.end,
            bucket_ms,
        )
        .expect("replay range")
    }

    #[test]
    fn range_replay_keeps_final_context_when_intersecting_chunk_contains_old_context() {
        let context_chunk = stored_context_chunk(
            1,
            0,
            vec![context_at(1, 0, "initial"), context_at(2, 1_000, "stale")],
        );
        let combat_chunk = stored_combat_chunk(
            1,
            0,
            vec![metric_hit_at(3, 100, HistoryMetric::Damage, 5, 0)],
        );
        let mut seed = HistoryProjectionReducer::new(0..0, 1).expect("seed reducer");
        seed.seed_contexts([HistoryEntityContext {
            entity_id: 1,
            display_uid: 1,
            name: Some("final".to_string()),
            class_id: None,
            class_spec: None,
            ability_score: None,
            season_strength: None,
            monster_id: None,
        }]);
        let mut snapshot = seed.finish_detail(1, empty_summary(1));
        snapshot.last_sequence = 3;

        let range = replay_chunks(1, 0, &snapshot, &[context_chunk, combat_chunk], 0, 500, 500)
            .expect("range replay");

        assert_eq!(range.entities[0].name.as_deref(), Some("final"));
    }

    #[test]
    fn adjacent_ranges_include_death_replays_by_death_offset_only() {
        let first_replay = death_replay(10_999);
        let second_replay = death_replay(11_000);
        let chunk = stored_combat_chunk(
            1,
            0,
            vec![
                death(1, 999, Some(first_replay.clone())),
                death(2, 1_000, Some(second_replay.clone())),
                death(3, 1_500, None),
            ],
        );

        let left = replay_range(0..1_000, std::slice::from_ref(&chunk));
        let right = replay_range(1_000..2_000, std::slice::from_ref(&chunk));

        assert_eq!(left.entities[0].deaths.len(), 1);
        assert_eq!(
            left.entities[0].deaths[0].replay,
            Some(DeathRecord::from(&first_replay))
        );
        assert_eq!(right.entities[0].deaths.len(), 2);
        assert_eq!(
            right.entities[0].deaths[0].replay,
            Some(DeathRecord::from(&second_replay))
        );
        assert!(right.entities[0].deaths[1].replay.is_none());
    }

    fn decimal(value: &str) -> u128 {
        value.parse().expect("decimal value")
    }

    fn assert_decimal_additive(whole: &str, left: &str, right: &str) {
        assert_eq!(decimal(whole), decimal(left) + decimal(right));
    }

    fn stats_for(range: &EncounterRangeData, metric: HistoryMetric) -> EncounterStatsData {
        range.entities[0]
            .skills
            .iter()
            .find(|skill| skill.metric == metric)
            .map(|skill| skill.stats.clone())
            .unwrap_or_else(|| EncounterStatsData {
                total: "0".to_string(),
                effective_total: "0".to_string(),
                critical_total: "0".to_string(),
                lucky_total: "0".to_string(),
                ..EncounterStatsData::default()
            })
    }

    fn target_stats_for(range: &EncounterRangeData, metric: HistoryMetric) -> EncounterStatsData {
        let Some(entity) = range.entities.first() else {
            return EncounterStatsData::default();
        };
        let targets = match metric {
            HistoryMetric::Damage => &entity.damage_targets,
            HistoryMetric::Healing => &entity.healing_targets,
            HistoryMetric::DamageTaken => return EncounterStatsData::default(),
        };
        targets
            .first()
            .map(|target| target.stats.clone())
            .unwrap_or_default()
    }

    fn source_stats_for(range: &EncounterRangeData) -> EncounterStatsData {
        range
            .entities
            .first()
            .and_then(|entity| entity.taken_sources.first())
            .map(|source| source.stats.clone())
            .unwrap_or_default()
    }

    fn assert_stats_additive(
        whole: &EncounterStatsData,
        left: &EncounterStatsData,
        right: &EncounterStatsData,
    ) {
        assert_decimal_additive(&whole.total, &left.total, &right.total);
        assert_decimal_additive(
            &whole.effective_total,
            &left.effective_total,
            &right.effective_total,
        );
        assert_decimal_additive(&whole.hits, &left.hits, &right.hits);
        assert_decimal_additive(
            &whole.critical_hits,
            &left.critical_hits,
            &right.critical_hits,
        );
        assert_decimal_additive(
            &whole.critical_total,
            &left.critical_total,
            &right.critical_total,
        );
        assert_decimal_additive(&whole.lucky_hits, &left.lucky_hits, &right.lucky_hits);
        assert_decimal_additive(&whole.lucky_total, &left.lucky_total, &right.lucky_total);
        assert_decimal_additive(&whole.trigger_hits, &left.trigger_hits, &right.trigger_hits);
        assert_decimal_additive(&whole.blocked_hits, &left.blocked_hits, &right.blocked_hits);
        assert_decimal_additive(
            &whole.lucky_block_hits,
            &left.lucky_block_hits,
            &right.lucky_block_hits,
        );
    }

    #[test]
    fn adjacent_half_open_ranges_are_additive_across_chunks() {
        let events = vec![
            metric_hit_at(0, 0, HistoryMetric::Damage, 5, CombatHitFlags::CRITICAL),
            metric_hit_at(1, 999, HistoryMetric::Healing, 8, 0),
            metric_hit_at(
                2,
                1_000,
                HistoryMetric::Damage,
                11,
                CombatHitFlags::ATTACKER_LUCKY,
            ),
            metric_hit_at(
                3,
                1_000,
                HistoryMetric::Healing,
                14,
                CombatHitFlags::ATTACKER_LUCKY,
            ),
            metric_hit_at(
                4,
                1_999,
                HistoryMetric::DamageTaken,
                17,
                CombatHitFlags::BLOCKED | CombatHitFlags::DEFENDER_LUCKY,
            ),
            metric_hit_at(5, 2_000, HistoryMetric::Damage, 19, 0),
        ];
        let chunks = vec![
            stored_combat_chunk(1, 0, events[..2].to_vec()),
            stored_combat_chunk(1, 1, events[2..].to_vec()),
        ];

        let whole = replay_range(0..2_000, &chunks);
        let left = replay_range(0..1_000, &chunks);
        let right = replay_range(1_000..2_000, &chunks);

        for totals in [
            (
                &whole.totals.damage,
                &left.totals.damage,
                &right.totals.damage,
            ),
            (
                &whole.totals.boss_damage,
                &left.totals.boss_damage,
                &right.totals.boss_damage,
            ),
            (
                &whole.totals.healing,
                &left.totals.healing,
                &right.totals.healing,
            ),
            (
                &whole.totals.effective_healing,
                &left.totals.effective_healing,
                &right.totals.effective_healing,
            ),
            (
                &whole.totals.damage_taken,
                &left.totals.damage_taken,
                &right.totals.damage_taken,
            ),
        ] {
            assert_decimal_additive(totals.0, totals.1, totals.2);
        }

        for metric in [
            HistoryMetric::Damage,
            HistoryMetric::Healing,
            HistoryMetric::DamageTaken,
        ] {
            assert_stats_additive(
                &stats_for(&whole, metric),
                &stats_for(&left, metric),
                &stats_for(&right, metric),
            );
        }
        for metric in [HistoryMetric::Damage, HistoryMetric::Healing] {
            assert_stats_additive(
                &target_stats_for(&whole, metric),
                &target_stats_for(&left, metric),
                &target_stats_for(&right, metric),
            );
        }
        assert_stats_additive(
            &source_stats_for(&whole),
            &source_stats_for(&left),
            &source_stats_for(&right),
        );
        assert_eq!(whole.totals.damage, "16");
        assert_eq!(whole.totals.healing, "22");
        assert_eq!(whole.totals.effective_healing, "11");
        assert_eq!(whole.totals.damage_taken, "17");
        assert_eq!(left.end_ms_exclusive, right.start_ms);
    }

    #[test]
    fn frozen_clock_offsets_have_no_pause_gap_and_remain_additive() {
        // These two events were captured six wall-clock seconds apart, but the
        // segment clock supplied to history froze for five paused seconds.
        let chunk = stored_combat_chunk(
            1,
            0,
            vec![
                metric_hit_at(0, 1_000, HistoryMetric::Damage, 10, 0),
                metric_hit_at(1, 2_000, HistoryMetric::Damage, 20, 0),
            ],
        );

        let whole = replay_range(1_000..2_001, std::slice::from_ref(&chunk));
        let left = replay_range(1_000..2_000, std::slice::from_ref(&chunk));
        let right = replay_range(2_000..2_001, &[chunk]);

        assert_eq!(whole.totals.damage, "30");
        assert_eq!(left.totals.damage, "10");
        assert_eq!(right.totals.damage, "20");
        assert_decimal_additive(
            &whole.totals.damage,
            &left.totals.damage,
            &right.totals.damage,
        );
        assert_eq!(whole.end_ms_exclusive - whole.start_ms, 1_001);
    }

    #[test]
    fn projection_codec_round_trips() {
        let mut reducer = HistoryProjectionReducer::new(0..10, 1).expect("valid reducer");
        reducer.apply(&hit(0, 1, 42));
        let snapshot = reducer.finish_detail(9, empty_summary(9));
        let encoded = encode_detail_projection(&snapshot).expect("encode projection");
        let decoded = decode_detail_projection(&encoded.data).expect("decode projection");
        assert_eq!(decoded.detail.totals, snapshot.detail.totals);
        assert_eq!(decoded.last_sequence, snapshot.last_sequence);
        assert_eq!(HistoryStream::Combat.as_db_str(), "combat");
    }

    #[test]
    fn projection_metadata_merges_external_quality_and_rejects_divergence() {
        let mut reducer = HistoryProjectionReducer::new(0..10, 1).expect("valid reducer");
        reducer.apply(&hit(0, 1, 42));
        let snapshot = reducer.finish_detail(9, empty_summary(9));
        let encoded = encode_detail_projection(&snapshot).expect("encode projection");
        let external_quality = quality_flags_to_bits(&[HistoryQualityFlag::IncompleteSegment]);
        let combined_quality = encoded.quality_flags | external_quality;
        let descriptor = EncounterHistoryDescriptor {
            encounter_id: 9,
            quality_flags: combined_quality,
            started_at_ms: 0,
            ended_at_ms: Some(10),
        };
        let mut stored = StoredProjection {
            encounter_id: 9,
            last_sequence: encoded.last_sequence,
            quality_flags: combined_quality,
            data: encoded.data,
        };

        assert_eq!(
            validate_projection_metadata(&descriptor, &stored, &snapshot)
                .expect("consistent metadata"),
            combined_quality
        );

        stored.quality_flags = encoded.quality_flags;
        assert!(matches!(
            validate_projection_metadata(&descriptor, &stored, &snapshot),
            Err(HistoryQueryError::ProjectionQualityMismatch { encounter_id: 9 })
        ));
    }

    #[test]
    fn canonical_metric_updates_exactly_one_combat_side() {
        let mut reducer = HistoryProjectionReducer::new(0..10, 10).expect("reducer");
        reducer.seed_contexts([HistoryEntityContext {
            entity_id: 1,
            display_uid: 1,
            name: Some("player".to_string()),
            class_id: Some(1),
            class_spec: None,
            ability_score: None,
            season_strength: None,
            monster_id: None,
        }]);
        reducer.apply(&metric_hit(1, HistoryMetric::Damage, 100, 0));
        reducer.apply(&metric_hit(2, HistoryMetric::Healing, 40, 0));
        reducer.apply(&metric_hit(3, HistoryMetric::DamageTaken, 30, 0));

        let range = reducer.finish_range(1);
        assert_eq!(range.totals.damage, "100");
        assert_eq!(range.totals.healing, "40");
        assert_eq!(range.totals.effective_healing, "20");
        assert_eq!(range.totals.damage_taken, "30");
        let player = range
            .entities
            .iter()
            .find(|entity| entity.entity_id == "1")
            .expect("player projection");
        assert_eq!(player.totals.damage, "100");
        assert_eq!(player.totals.healing, "40");
        assert_eq!(player.totals.damage_taken, "30");
        assert!(range.entities.iter().all(|entity| entity.entity_id != "2"));
    }

    #[test]
    fn range_projects_target_and_source_breakdowns_without_top_level_pollution() {
        let mut reducer = HistoryProjectionReducer::new(0..1_000, 1_000).expect("reducer");
        reducer.seed_contexts([
            HistoryEntityContext {
                entity_id: 1,
                display_uid: 101,
                name: Some("player".to_string()),
                class_id: Some(1),
                class_spec: None,
                ability_score: None,
                season_strength: None,
                monster_id: None,
            },
            HistoryEntityContext {
                entity_id: 2,
                display_uid: 202,
                name: Some("boss".to_string()),
                class_id: None,
                class_spec: None,
                ability_score: None,
                season_strength: None,
                monster_id: Some(30_001),
            },
            HistoryEntityContext {
                entity_id: 3,
                display_uid: 303,
                name: Some("ally".to_string()),
                class_id: Some(2),
                class_spec: None,
                ability_score: None,
                season_strength: None,
                monster_id: None,
            },
        ]);

        let mut damage = metric_hit_at(
            1,
            100,
            HistoryMetric::Damage,
            120,
            CombatHitFlags::CRITICAL | CombatHitFlags::ATTACKER_LUCKY,
        );
        let HistoryEvent::Hit(damage_hit) = &mut damage.event else {
            unreachable!("metric helper creates a hit")
        };
        damage_hit.target_entity_id = 2;
        damage_hit.target_is_boss = true;
        damage_hit.target_monster_id = Some(30_001);
        damage_hit.base_skill_id = Some(1_714);

        let mut healing =
            metric_hit_at(2, 200, HistoryMetric::Healing, 80, CombatHitFlags::CRITICAL);
        let HistoryEvent::Hit(healing_hit) = &mut healing.event else {
            unreachable!("metric helper creates a hit")
        };
        healing_hit.target_entity_id = 3;
        healing_hit.hp_loss = 0;

        let mut taken = metric_hit_at(
            3,
            300,
            HistoryMetric::DamageTaken,
            60,
            CombatHitFlags::BLOCKED | CombatHitFlags::DEFENDER_LUCKY,
        );
        let HistoryEvent::Hit(taken_hit) = &mut taken.event else {
            unreachable!("metric helper creates a hit")
        };
        taken_hit.source_monster_id = Some(9_001);

        reducer.apply(&damage);
        reducer.apply(&healing);
        reducer.apply(&taken);
        let range = reducer.finish_range(7);

        assert_eq!(range.entities.len(), 1, "targets are not top-level actors");
        let player = &range.entities[0];
        let damage_target = &player.damage_targets[0];
        assert_eq!(damage_target.target_entity_id, "2");
        assert_eq!(damage_target.target_display_uid, 202);
        assert_eq!(damage_target.target_name.as_deref(), Some("boss"));
        assert_eq!(damage_target.target_monster_id, Some(30_001));
        assert!(damage_target.is_boss);
        assert_eq!(damage_target.stats.total, "120");
        assert_eq!(damage_target.stats.critical_hits, "1");
        assert_eq!(damage_target.skills[0].stats.lucky_hits, "1");

        let healing_target = &player.healing_targets[0];
        assert_eq!(healing_target.target_entity_id, "3");
        assert_eq!(healing_target.target_name.as_deref(), Some("ally"));
        assert_eq!(healing_target.stats.total, "80");
        assert_eq!(healing_target.stats.effective_total, "40");

        let source = &player.taken_sources[0];
        assert_eq!(source.source_monster_id, Some(9_001));
        assert_eq!(source.stats.total, "60");
        assert_eq!(source.stats.blocked_hits, "1");
        assert_eq!(source.stats.lucky_hits, "1");
        assert_eq!(source.skills[0].metric, HistoryMetric::DamageTaken);
    }

    #[test]
    fn lucky_bonus_packets_do_not_count_as_triggers() {
        let mut reducer = HistoryProjectionReducer::new(0..10, 10).expect("reducer");
        reducer.apply(&metric_hit(
            1,
            HistoryMetric::Damage,
            100,
            CombatHitFlags::ATTACKER_LUCKY,
        ));
        reducer.apply(&metric_hit(
            2,
            HistoryMetric::Damage,
            20,
            CombatHitFlags::LUCKY_BONUS_ONLY,
        ));

        let range = reducer.finish_range(1);
        let stats = &range.entities[0].skills[0].stats;
        assert_eq!(stats.total, "120");
        assert_eq!(stats.hits, "2");
        assert_eq!(stats.trigger_hits, "1");
        assert_eq!(stats.lucky_hits, "1");
        assert_eq!(stats.lucky_total, "20");
    }

    #[test]
    fn chart_bucket_ceil_divides_duration_by_target_points() {
        assert_eq!(
            bucket_width_for_points(0, 10_001, 10).expect("bucket"),
            1_001
        );
        assert_eq!(
            bucket_width_for_points(5_000, 5_001, 500).expect("bucket"),
            1
        );
        assert!(matches!(
            bucket_width_for_points(0, 1_000, 0),
            Err(HistoryQueryError::ZeroTargetPoints)
        ));
    }

    fn actor_hit(
        sequence: u64,
        offset_ms: u64,
        actor_entity_id: i64,
        metric: HistoryMetric,
        amount: u64,
    ) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms,
            event: HistoryEvent::Hit(HistoryHit {
                actor_entity_id,
                source_entity_id: Some(actor_entity_id),
                target_entity_id: if metric == HistoryMetric::DamageTaken {
                    actor_entity_id
                } else {
                    99
                },
                skill_id: 7,
                base_skill_id: Some(7),
                metric,
                amount: amount.into(),
                effective_amount: amount.into(),
                has_loss_breakdown: metric == HistoryMetric::DamageTaken,
                hp_loss: amount.into(),
                shield_loss: 0,
                flags: 0,
                target_is_boss: false,
                target_monster_id: None,
                source_monster_id: None,
                property: None,
                damage_mode: None,
            }),
        }
    }

    fn series_row<'a>(
        range: &'a EncounterRangeData,
        entity_id: &str,
        metric: HistoryMetric,
    ) -> Option<&'a EncounterChartSeriesData> {
        range
            .series
            .iter()
            .find(|row| row.entity_id == entity_id && row.metric == metric)
    }

    #[test]
    fn per_entity_series_track_actors_and_sum_to_team_chart() {
        let mut reducer = HistoryProjectionReducer::new(0..10_000, 1_000).expect("reducer");
        reducer.apply(&actor_hit(1, 100, 1, HistoryMetric::Damage, 100));
        reducer.apply(&actor_hit(2, 1_500, 1, HistoryMetric::Damage, 150));
        reducer.apply(&actor_hit(3, 100, 2, HistoryMetric::Damage, 300));
        reducer.apply(&actor_hit(4, 200, 2, HistoryMetric::Healing, 50));

        let range = reducer.finish_range(1);

        let actor_one = series_row(&range, "1", HistoryMetric::Damage).expect("actor 1 damage");
        assert_eq!(actor_one.offsets_ms, vec![0, 1_000]);
        assert_eq!(actor_one.totals, vec!["100".to_string(), "150".to_string()]);
        let actor_two = series_row(&range, "2", HistoryMetric::Damage).expect("actor 2 damage");
        assert_eq!(actor_two.offsets_ms, vec![0]);
        assert_eq!(actor_two.totals, vec!["300".to_string()]);
        let actor_two_healing =
            series_row(&range, "2", HistoryMetric::Healing).expect("actor 2 healing");
        assert_eq!(actor_two_healing.totals, vec!["50".to_string()]);
        // Sparse: metrics without any hit emit no row at all.
        assert!(series_row(&range, "1", HistoryMetric::Healing).is_none());
        assert!(series_row(&range, "1", HistoryMetric::DamageTaken).is_none());
        assert!(series_row(&range, "2", HistoryMetric::DamageTaken).is_none());

        // The per-entity series partition the team chart exactly.
        let team_damage: u128 = range
            .chart_points
            .iter()
            .map(|point| decimal(&point.damage))
            .sum();
        let entity_damage: u128 = range
            .series
            .iter()
            .filter(|row| row.metric == HistoryMetric::Damage)
            .flat_map(|row| row.totals.iter().map(|total| decimal(total)))
            .sum();
        assert_eq!(team_damage, entity_damage);
        let bucket_zero = &range.chart_points[0];
        assert_eq!(bucket_zero.damage, "400");
        assert_eq!(bucket_zero.healing, "50");
    }

    #[test]
    fn stored_projection_snapshot_drops_dynamic_series() {
        let mut reducer = HistoryProjectionReducer::new(0..10_000, 1_000).expect("reducer");
        reducer.apply(&actor_hit(1, 100, 1, HistoryMetric::Damage, 100));

        let snapshot = reducer.finish_detail(1, empty_summary(1));

        assert!(snapshot.detail.chart_points.is_empty());
        assert!(snapshot.detail.series.is_empty());
        assert!(snapshot.detail.markers.is_empty());
    }

    #[test]
    fn context_class_spec_discriminant_resolves_spec_name() {
        let mut reducer = HistoryProjectionReducer::new(0..10, 10).expect("reducer");
        reducer.seed_contexts([HistoryEntityContext {
            entity_id: 1,
            display_uid: 1,
            name: Some("player".to_string()),
            class_id: Some(12),
            class_spec: Some(15), // ClassSpec::Recovery discriminant
            ability_score: None,
            season_strength: None,
            monster_id: None,
        }]);
        reducer.apply(&actor_hit(1, 0, 1, HistoryMetric::Damage, 100));

        let range = reducer.finish_range(1);

        let entity = &range.entities[0];
        assert_eq!(entity.class_spec, Some(15));
        assert_eq!(entity.class_spec_name.as_deref(), Some("Recovery"));
    }
}
