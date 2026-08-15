//! Stable persistence schema and bounded chunk assembly for historical facts.
//!
//! Runtime domain events are converted explicitly into [`HistoryEvent`]. This
//! module never serializes the runtime event enum itself, so adding a live-only
//! event cannot silently change the on-disk contract.

use serde::{Deserialize, Serialize};

use crate::live::projections::combat::accumulator::{CombatHitFact, CombatHitFlags, CombatMetric};
use crate::live::projections::death::DeathReplaySnapshot;

pub const MAX_EVENTS_PER_CHUNK: usize = 8_192;
pub const TARGET_UNCOMPRESSED_CHUNK_BYTES: usize = 1024 * 1024;
pub const MAX_CHUNK_SPAN_MS: u64 = 2_000;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, specta::Type,
)]
#[serde(rename_all = "snake_case")]
pub enum HistoryStream {
    Combat,
    Timeline,
    Context,
}

impl HistoryStream {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::Combat => "combat",
            Self::Timeline => "timeline",
            Self::Context => "context",
        }
    }

    pub fn from_db_str(value: &str) -> Result<Self, HistoryCodecError> {
        match value {
            "combat" => Ok(Self::Combat),
            "timeline" => Ok(Self::Timeline),
            "context" => Ok(Self::Context),
            _ => Err(HistoryCodecError::InvalidStream(value.to_string())),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, specta::Type,
)]
#[serde(rename_all = "snake_case")]
pub enum HistoryMetric {
    Damage,
    Healing,
    DamageTaken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryHit {
    pub actor_entity_id: i64,
    pub source_entity_id: Option<i64>,
    pub target_entity_id: i64,
    pub skill_id: i64,
    pub base_skill_id: Option<i32>,
    pub metric: HistoryMetric,
    pub amount: u128,
    pub effective_amount: u128,
    pub has_loss_breakdown: bool,
    pub hp_loss: u128,
    pub shield_loss: u128,
    pub flags: u8,
    pub target_is_boss: bool,
    pub target_monster_id: Option<i32>,
    pub source_monster_id: Option<i32>,
    pub property: Option<i32>,
    pub damage_mode: Option<i32>,
}

impl From<CombatMetric> for HistoryMetric {
    fn from(value: CombatMetric) -> Self {
        match value {
            CombatMetric::Damage => Self::Damage,
            CombatMetric::Healing => Self::Healing,
            CombatMetric::DamageTaken => Self::DamageTaken,
        }
    }
}

impl From<HistoryMetric> for CombatMetric {
    fn from(value: HistoryMetric) -> Self {
        match value {
            HistoryMetric::Damage => Self::Damage,
            HistoryMetric::Healing => Self::Healing,
            HistoryMetric::DamageTaken => Self::DamageTaken,
        }
    }
}

impl From<&CombatHitFact> for HistoryHit {
    fn from(hit: &CombatHitFact) -> Self {
        Self {
            actor_entity_id: hit.actor_entity_id,
            source_entity_id: hit.source_entity_id,
            target_entity_id: hit.target_entity_id,
            skill_id: hit.skill_key,
            base_skill_id: hit.base_skill_id,
            metric: hit.metric.into(),
            amount: hit.amount,
            effective_amount: hit.effective_amount,
            has_loss_breakdown: hit.has_loss_breakdown,
            hp_loss: hit.hp_loss,
            shield_loss: hit.shield_loss,
            flags: hit.flags.bits(),
            target_is_boss: hit.target_is_boss,
            target_monster_id: hit.target_monster_id,
            source_monster_id: hit.source_monster_id,
            property: hit.property,
            damage_mode: hit.damage_mode,
        }
    }
}

impl From<&HistoryHit> for CombatHitFact {
    fn from(hit: &HistoryHit) -> Self {
        Self {
            metric: hit.metric.into(),
            actor_entity_id: hit.actor_entity_id,
            source_entity_id: hit.source_entity_id,
            target_entity_id: hit.target_entity_id,
            source_monster_id: hit.source_monster_id,
            target_monster_id: hit.target_monster_id,
            target_is_boss: hit.target_is_boss,
            skill_key: hit.skill_id,
            base_skill_id: hit.base_skill_id,
            amount: hit.amount,
            effective_amount: hit.effective_amount,
            has_loss_breakdown: hit.has_loss_breakdown,
            hp_loss: hit.hp_loss,
            shield_loss: hit.shield_loss,
            flags: CombatHitFlags::from_bits(hit.flags),
            property: hit.property,
            damage_mode: hit.damage_mode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum HistoryCastKind {
    BossSkill,
    Fantasy,
    KeySkill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySkillCast {
    pub caster_entity_id: i64,
    pub skill_id: i64,
    pub kind: HistoryCastKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntityContext {
    pub entity_id: i64,
    pub display_uid: i64,
    pub name: Option<String>,
    pub class_id: Option<i32>,
    pub class_spec: Option<i32>,
    pub ability_score: Option<i32>,
    pub season_strength: Option<i32>,
    pub monster_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryDeath {
    pub entity_id: i64,
    pub source_entity_id: Option<i64>,
    pub skill_id: Option<i64>,
    pub replay: Option<DeathReplaySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum HistoryEvent {
    Hit(HistoryHit),
    SkillCast(HistorySkillCast),
    EntityContext(HistoryEntityContext),
    Death(HistoryDeath),
}

impl HistoryEvent {
    pub const fn stream_kind(&self) -> HistoryStream {
        match self {
            Self::Hit(_) | Self::Death(_) => HistoryStream::Combat,
            Self::SkillCast(_) => HistoryStream::Timeline,
            Self::EntityContext(_) => HistoryStream::Context,
        }
    }

    fn estimated_encoded_size(&self) -> usize {
        match self {
            // Named MessagePack repeats the hit field names for every event.
            // Include wide integer values and the sequence/offset columns in
            // the estimate so the encoded document stays near the 1 MiB goal.
            Self::Hit(_) => 384,
            Self::SkillCast(_) => 56,
            Self::Death(death) => estimated_death_size(death),
            Self::EntityContext(context) => 96 + context.name.as_ref().map_or(0, String::len),
        }
    }
}

fn estimated_death_size(death: &HistoryDeath) -> usize {
    let Some(replay) = &death.replay else {
        return 64;
    };
    let participant_buff_count = replay
        .participant_buffs
        .iter()
        .map(|participant| participant.buffs.len())
        .sum::<usize>();
    128usize
        .saturating_add(replay.recent_damages.len().saturating_mul(96))
        .saturating_add(replay.victim_buffs.len().saturating_mul(96))
        .saturating_add(replay.participant_buffs.len().saturating_mul(64))
        .saturating_add(participant_buff_count.saturating_mul(96))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEnvelope {
    /// Segment-local, globally ordered sequence number.
    pub sequence: u64,
    /// Event time relative to the segment start.
    pub offset_ms: u64,
    pub event: HistoryEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryChunkDocument {
    pub stream_kind: HistoryStream,
    pub sequences: Vec<u64>,
    pub offsets_ms: Vec<u64>,
    pub events: Vec<HistoryEvent>,
}

impl HistoryChunkDocument {
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn envelopes(&self) -> impl Iterator<Item = HistoryEnvelope> + '_ {
        self.sequences
            .iter()
            .copied()
            .zip(self.offsets_ms.iter().copied())
            .zip(self.events.iter().cloned())
            .map(|((sequence, offset_ms), event)| HistoryEnvelope {
                sequence,
                offset_ms,
                event,
            })
    }

    fn validate(&self) -> Result<(), HistoryCodecError> {
        let len = self.events.len();
        if len == 0 || self.sequences.len() != len || self.offsets_ms.len() != len {
            return Err(HistoryCodecError::InvalidDocument(
                "chunk columns have different lengths or are empty".to_string(),
            ));
        }
        if self.sequences.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(HistoryCodecError::InvalidDocument(
                "chunk sequences are not strictly increasing".to_string(),
            ));
        }
        if self
            .events
            .iter()
            .any(|event| event.stream_kind() != self.stream_kind)
        {
            return Err(HistoryCodecError::InvalidDocument(
                "chunk contains an event from another stream".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedHistoryChunk {
    pub encounter_id: i32,
    pub stream_kind: HistoryStream,
    pub chunk_index: u64,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub start_offset_ms: u64,
    pub end_offset_ms_exclusive: u64,
    pub event_count: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum HistoryCodecError {
    #[error("invalid history stream {0}")]
    InvalidStream(String),
    #[error("history events must have strictly increasing segment sequences")]
    NonMonotonicSequence,
    #[error("history sequence or time range overflowed")]
    RangeOverflow,
    #[error("invalid history chunk: {0}")]
    InvalidDocument(String),
    #[error("history chunk serialization failed: {0}")]
    Encode(String),
    #[error("history chunk decompression failed: {0}")]
    Decode(String),
}

#[derive(Debug, Default)]
struct StreamBuffer {
    next_chunk_index: u64,
    estimated_bytes: usize,
    min_offset_ms: Option<u64>,
    max_offset_ms: Option<u64>,
    events: Vec<HistoryEnvelope>,
}

impl StreamBuffer {
    fn should_seal_before(&self, envelope: &HistoryEnvelope) -> bool {
        let (Some(min_offset), Some(max_offset)) = (self.min_offset_ms, self.max_offset_ms) else {
            return false;
        };
        let expanded_min = min_offset.min(envelope.offset_ms);
        let expanded_max = max_offset.max(envelope.offset_ms);
        self.events.len() >= MAX_EVENTS_PER_CHUNK
            || self
                .estimated_bytes
                .saturating_add(envelope.event.estimated_encoded_size())
                > TARGET_UNCOMPRESSED_CHUNK_BYTES
            || expanded_max.saturating_sub(expanded_min) >= MAX_CHUNK_SPAN_MS
    }
}

/// Incrementally seals independent combat/timeline/context streams.
#[derive(Debug)]
pub struct HistoryChunker {
    encounter_id: i32,
    last_sequence: Option<u64>,
    combat: StreamBuffer,
    timeline: StreamBuffer,
    context: StreamBuffer,
}

impl HistoryChunker {
    pub fn new(encounter_id: i32) -> Result<Self, HistoryCodecError> {
        if encounter_id <= 0 {
            return Err(HistoryCodecError::InvalidDocument(
                "encounter id must be positive".to_string(),
            ));
        }
        Ok(Self {
            encounter_id,
            last_sequence: None,
            combat: StreamBuffer::default(),
            timeline: StreamBuffer::default(),
            context: StreamBuffer::default(),
        })
    }

    /// Push one event and return a sealed chunk when a bound is reached.
    pub fn push(
        &mut self,
        envelope: HistoryEnvelope,
    ) -> Result<Option<EncodedHistoryChunk>, HistoryCodecError> {
        if self
            .last_sequence
            .is_some_and(|last| envelope.sequence <= last)
        {
            return Err(HistoryCodecError::NonMonotonicSequence);
        }
        self.last_sequence = Some(envelope.sequence);

        let stream_kind = envelope.event.stream_kind();
        let should_seal = self.buffer(stream_kind).should_seal_before(&envelope);
        let sealed = should_seal
            .then(|| self.seal_stream(stream_kind))
            .transpose()?;
        let buffer = self.buffer_mut(stream_kind);
        buffer.estimated_bytes = buffer
            .estimated_bytes
            .saturating_add(envelope.event.estimated_encoded_size());
        buffer.min_offset_ms = Some(
            buffer
                .min_offset_ms
                .map_or(envelope.offset_ms, |value| value.min(envelope.offset_ms)),
        );
        buffer.max_offset_ms = Some(
            buffer
                .max_offset_ms
                .map_or(envelope.offset_ms, |value| value.max(envelope.offset_ms)),
        );
        buffer.events.push(envelope);
        Ok(sealed.flatten())
    }

    /// Seal all currently buffered streams without ending the encounter.
    /// The writer uses this when a buffer has been open for two wall-clock
    /// seconds but no newer event arrived to trigger span-based sealing.
    pub fn flush(&mut self) -> Result<Vec<EncodedHistoryChunk>, HistoryCodecError> {
        let mut chunks = Vec::with_capacity(3);
        for stream in [
            HistoryStream::Combat,
            HistoryStream::Timeline,
            HistoryStream::Context,
        ] {
            if let Some(chunk) = self.seal_stream(stream)? {
                chunks.push(chunk);
            }
        }
        chunks.sort_unstable_by(|left, right| {
            left.first_sequence
                .cmp(&right.first_sequence)
                .then_with(|| left.stream_kind.cmp(&right.stream_kind))
        });
        Ok(chunks)
    }

    fn buffer(&self, stream_kind: HistoryStream) -> &StreamBuffer {
        match stream_kind {
            HistoryStream::Combat => &self.combat,
            HistoryStream::Timeline => &self.timeline,
            HistoryStream::Context => &self.context,
        }
    }

    fn buffer_mut(&mut self, stream_kind: HistoryStream) -> &mut StreamBuffer {
        match stream_kind {
            HistoryStream::Combat => &mut self.combat,
            HistoryStream::Timeline => &mut self.timeline,
            HistoryStream::Context => &mut self.context,
        }
    }

    fn seal_stream(
        &mut self,
        stream_kind: HistoryStream,
    ) -> Result<Option<EncodedHistoryChunk>, HistoryCodecError> {
        let encounter_id = self.encounter_id;
        let buffer = self.buffer_mut(stream_kind);
        if buffer.events.is_empty() {
            return Ok(None);
        }

        let events = std::mem::take(&mut buffer.events);
        buffer.estimated_bytes = 0;
        buffer.min_offset_ms = None;
        buffer.max_offset_ms = None;
        let chunk_index = buffer.next_chunk_index;
        buffer.next_chunk_index = buffer
            .next_chunk_index
            .checked_add(1)
            .ok_or(HistoryCodecError::RangeOverflow)?;
        encode_history_chunk(encounter_id, stream_kind, chunk_index, events).map(Some)
    }
}

pub fn encode_history_chunk(
    encounter_id: i32,
    stream_kind: HistoryStream,
    chunk_index: u64,
    events: Vec<HistoryEnvelope>,
) -> Result<EncodedHistoryChunk, HistoryCodecError> {
    let first = events.first().ok_or_else(|| {
        HistoryCodecError::InvalidDocument("cannot encode an empty chunk".to_string())
    })?;
    if events
        .windows(2)
        .any(|pair| pair[0].sequence >= pair[1].sequence)
        || events
            .iter()
            .any(|item| item.event.stream_kind() != stream_kind)
    {
        return Err(HistoryCodecError::InvalidDocument(
            "chunk sequence or stream invariant failed".to_string(),
        ));
    }

    let first_sequence = first.sequence;
    let last_sequence = events
        .last()
        .map(|event| event.sequence)
        .ok_or(HistoryCodecError::RangeOverflow)?;
    let start_offset_ms = events
        .iter()
        .map(|event| event.offset_ms)
        .min()
        .ok_or(HistoryCodecError::RangeOverflow)?;
    let end_offset_ms_exclusive = events
        .iter()
        .map(|event| event.offset_ms)
        .max()
        .and_then(|value| value.checked_add(1))
        .ok_or(HistoryCodecError::RangeOverflow)?;
    let event_count = u64::try_from(events.len()).map_err(|_| HistoryCodecError::RangeOverflow)?;
    let document = HistoryChunkDocument {
        stream_kind,
        sequences: events.iter().map(|event| event.sequence).collect(),
        offsets_ms: events.iter().map(|event| event.offset_ms).collect(),
        events: events.into_iter().map(|event| event.event).collect(),
    };
    document.validate()?;
    let encoded = rmp_serde::to_vec_named(&document)
        .map_err(|error| HistoryCodecError::Encode(error.to_string()))?;
    let data = zstd::encode_all(&encoded[..], 3)
        .map_err(|error| HistoryCodecError::Encode(error.to_string()))?;

    Ok(EncodedHistoryChunk {
        encounter_id,
        stream_kind,
        chunk_index,
        first_sequence,
        last_sequence,
        start_offset_ms,
        end_offset_ms_exclusive,
        event_count,
        data,
    })
}

pub fn decode_history_chunk(
    data: &[u8],
    expected_stream_kind: HistoryStream,
) -> Result<HistoryChunkDocument, HistoryCodecError> {
    let decoded =
        zstd::decode_all(data).map_err(|error| HistoryCodecError::Decode(error.to_string()))?;
    let document: HistoryChunkDocument = rmp_serde::from_slice(&decoded)
        .map_err(|error| HistoryCodecError::Decode(error.to_string()))?;
    document.validate()?;
    if document.stream_kind != expected_stream_kind {
        return Err(HistoryCodecError::InvalidDocument(
            "row stream differs from document stream".to_string(),
        ));
    }
    Ok(document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::projections::combat::accumulator::CombatAccumulator;
    use crate::live::projections::combat::stats::class::ClassSpec;
    use crate::live::projections::death::{
        DeathReplayBuff, DeathReplayDamage, DeathReplayParticipant,
    };

    #[test]
    fn combat_fact_history_hit_round_trip_is_lossless() {
        let fact = CombatHitFact {
            metric: CombatMetric::DamageTaken,
            actor_entity_id: 22,
            source_entity_id: Some(11),
            target_entity_id: 22,
            source_monster_id: Some(9_001),
            target_monster_id: Some(30_001),
            target_is_boss: true,
            skill_key: 17_140_101,
            base_skill_id: Some(1_714),
            amount: u128::from(u64::MAX) + 7,
            effective_amount: 123,
            has_loss_breakdown: true,
            hp_loss: 80,
            shield_loss: 20,
            flags: CombatHitFlags::from_bits(
                CombatHitFlags::CRITICAL
                    | CombatHitFlags::BLOCKED
                    | CombatHitFlags::DEFENDER_LUCKY
                    | CombatHitFlags::LUCKY_BONUS_ONLY,
            ),
            property: Some(3),
            damage_mode: Some(4),
        };

        let stored = HistoryHit::from(&fact);
        assert_eq!(CombatHitFact::from(&stored), fact);
    }

    #[test]
    fn death_replay_history_chunk_round_trip_preserves_numeric_state() {
        let replay = DeathReplaySnapshot {
            victim_entity_uuid: 20,
            death_timestamp_ms: 3_000,
            recent_damages: vec![DeathReplayDamage {
                timestamp_ms: 2_500,
                attacker_entity_uuid: Some(10),
                attacker_monster_type_id: Some(9_001),
                skill_key: 17_140_101,
                value: u128::MAX,
            }],
            victim_buffs: vec![DeathReplayBuff {
                base_id: 77,
                instance_id: i64::MAX,
                layer: 2,
                duration_ms: None,
                started_wall_ms: Some(2_000),
                source_entity_uuid: Some(10),
                source_config_id: Some(700),
            }],
            participant_buffs: vec![DeathReplayParticipant {
                entity_uuid: Some(10),
                monster_type_id: Some(9_001),
                buffs: vec![DeathReplayBuff {
                    base_id: 44,
                    instance_id: 55,
                    layer: 1,
                    duration_ms: Some(5_000),
                    started_wall_ms: Some(1_500),
                    source_entity_uuid: None,
                    source_config_id: None,
                }],
            }],
        };
        let envelope = HistoryEnvelope {
            sequence: 7,
            offset_ms: 3_000,
            event: HistoryEvent::Death(HistoryDeath {
                entity_id: 20,
                source_entity_id: Some(10),
                skill_id: Some(17_140_101),
                replay: Some(replay),
            }),
        };

        let encoded = encode_history_chunk(1, HistoryStream::Combat, 0, vec![envelope.clone()])
            .expect("encode death replay");
        let decoded = decode_history_chunk(&encoded.data, HistoryStream::Combat)
            .expect("decode death replay");

        assert_eq!(decoded.envelopes().collect::<Vec<_>>(), vec![envelope]);
    }

    #[test]
    fn full_range_chunk_replay_matches_live_combat_accumulator() {
        let facts = [
            (
                0,
                CombatHitFact {
                    metric: CombatMetric::Damage,
                    actor_entity_id: 10,
                    source_entity_id: Some(10),
                    target_entity_id: 20,
                    source_monster_id: None,
                    target_monster_id: Some(30_001),
                    target_is_boss: true,
                    skill_key: 17_140_101,
                    base_skill_id: Some(1_714),
                    amount: 100,
                    effective_amount: 0,
                    has_loss_breakdown: true,
                    hp_loss: 70,
                    shield_loss: 30,
                    flags: CombatHitFlags::from_bits(
                        CombatHitFlags::CRITICAL | CombatHitFlags::ATTACKER_LUCKY,
                    ),
                    property: Some(1),
                    damage_mode: Some(2),
                },
            ),
            (
                250,
                CombatHitFact {
                    metric: CombatMetric::Damage,
                    actor_entity_id: 10,
                    source_entity_id: Some(10),
                    target_entity_id: 20,
                    source_monster_id: None,
                    target_monster_id: Some(30_001),
                    target_is_boss: true,
                    skill_key: 17_140_101,
                    base_skill_id: Some(1_714),
                    amount: 25,
                    effective_amount: 0,
                    has_loss_breakdown: true,
                    hp_loss: 25,
                    shield_loss: 0,
                    flags: CombatHitFlags::from_bits(CombatHitFlags::LUCKY_BONUS_ONLY),
                    property: Some(1),
                    damage_mode: Some(2),
                },
            ),
            (
                1_000,
                CombatHitFact {
                    metric: CombatMetric::Damage,
                    actor_entity_id: 10,
                    source_entity_id: Some(10),
                    target_entity_id: 21,
                    source_monster_id: None,
                    target_monster_id: Some(30_002),
                    target_is_boss: false,
                    skill_key: 17_140_201,
                    base_skill_id: Some(1_714),
                    amount: 40,
                    effective_amount: 0,
                    has_loss_breakdown: true,
                    hp_loss: 30,
                    shield_loss: 10,
                    flags: CombatHitFlags::default(),
                    property: Some(3),
                    damage_mode: Some(4),
                },
            ),
            (
                2_200,
                CombatHitFact {
                    metric: CombatMetric::Healing,
                    actor_entity_id: 11,
                    source_entity_id: Some(11),
                    target_entity_id: 10,
                    source_monster_id: None,
                    target_monster_id: None,
                    target_is_boss: false,
                    skill_key: 24_060_101,
                    base_skill_id: Some(2_406),
                    amount: 80,
                    effective_amount: 50,
                    has_loss_breakdown: false,
                    hp_loss: 0,
                    shield_loss: 0,
                    flags: CombatHitFlags::from_bits(
                        CombatHitFlags::CRITICAL | CombatHitFlags::ATTACKER_LUCKY,
                    ),
                    property: Some(5),
                    damage_mode: Some(6),
                },
            ),
            (
                3_500,
                CombatHitFact {
                    metric: CombatMetric::Healing,
                    actor_entity_id: 11,
                    source_entity_id: Some(11),
                    target_entity_id: 12,
                    source_monster_id: None,
                    target_monster_id: None,
                    target_is_boss: false,
                    skill_key: 24_060_101,
                    base_skill_id: Some(2_406),
                    amount: 20,
                    effective_amount: 20,
                    has_loss_breakdown: false,
                    hp_loss: 0,
                    shield_loss: 0,
                    flags: CombatHitFlags::from_bits(CombatHitFlags::LUCKY_BONUS_ONLY),
                    property: Some(5),
                    damage_mode: Some(6),
                },
            ),
            (
                4_999,
                CombatHitFact {
                    metric: CombatMetric::DamageTaken,
                    actor_entity_id: 10,
                    source_entity_id: Some(30),
                    target_entity_id: 10,
                    source_monster_id: Some(9_001),
                    target_monster_id: None,
                    target_is_boss: false,
                    skill_key: 50_010_101,
                    base_skill_id: None,
                    amount: 100,
                    effective_amount: 0,
                    has_loss_breakdown: true,
                    hp_loss: 60,
                    shield_loss: 40,
                    flags: CombatHitFlags::from_bits(
                        CombatHitFlags::CRITICAL
                            | CombatHitFlags::BLOCKED
                            | CombatHitFlags::DEFENDER_LUCKY,
                    ),
                    property: Some(7),
                    damage_mode: Some(8),
                },
            ),
        ];

        let mut live = CombatAccumulator::default();
        let mut chunker = HistoryChunker::new(42).expect("create chunker");
        let mut chunks = Vec::new();
        for (sequence, (offset_ms, fact)) in facts.iter().enumerate() {
            assert!(!live.apply(fact));
            let envelope = HistoryEnvelope {
                sequence: u64::try_from(sequence).expect("test sequence fits u64"),
                offset_ms: *offset_ms,
                event: HistoryEvent::Hit(HistoryHit::from(fact)),
            };
            if let Some(chunk) = chunker.push(envelope).expect("append history hit") {
                chunks.push(chunk);
            }
        }
        chunks.extend(chunker.flush().expect("flush history chunks"));
        assert!(chunks.len() >= 2, "test must exercise cross-chunk replay");

        let full_range = 0..5_000;
        let mut replayed = CombatAccumulator::default();
        for chunk in &chunks {
            let document =
                decode_history_chunk(&chunk.data, chunk.stream_kind).expect("decode history chunk");
            for envelope in document.envelopes() {
                if !full_range.contains(&envelope.offset_ms) {
                    continue;
                }
                let HistoryEvent::Hit(hit) = envelope.event else {
                    continue;
                };
                assert!(!replayed.apply(&CombatHitFact::from(&hit)));
            }
        }

        assert_eq!(live.totals.damage, 165);
        assert_eq!(live.totals.boss_damage, 125);
        assert_eq!(live.totals.healing, 100);
        assert_eq!(live.totals.effective_healing, 70);
        assert_eq!(live.totals.damage_taken, 100);

        let attacker = &live.entities[&10];
        assert_eq!(attacker.class_spec, ClassSpec::Iaido);
        assert_eq!(attacker.damage_boss_only.total, 125);
        assert_eq!(attacker.damage_targets[&20].stats.total_value, 125);
        assert_eq!(attacker.damage_targets[&21].stats.total_value, 40);
        let boss_skill = &attacker.damage_targets[&20].skills[&17_140_101];
        assert_eq!(boss_skill.hp_loss_total, 95);
        assert_eq!(boss_skill.shield_loss_total, 30);
        assert_eq!(boss_skill.crit_hits, 1);
        assert_eq!(boss_skill.lucky_hits, 1);
        assert_eq!(boss_skill.lucky_total, 25);
        assert_eq!(boss_skill.trigger_hits, 1);

        let healer = &live.entities[&11];
        assert_eq!(healer.class_spec, ClassSpec::Shield);
        assert_eq!(
            healer.healing_targets[&10].skills[&24_060_101].effective_total_value,
            50
        );
        assert_eq!(
            healer.healing_targets[&12].skills[&24_060_101].effective_total_value,
            20
        );

        let taken = &attacker.taken_sources[&Some(9_001)].skills[&50_010_101];
        assert_eq!(taken.crit_hits, 1);
        assert_eq!(taken.lucky_hits, 1);
        assert_eq!(taken.block_hits, 1);
        assert_eq!(taken.lucky_block_hits, 1);

        assert_eq!(replayed, live);
    }

    fn hit(sequence: u64, offset_ms: u64, amount: u64) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms,
            event: HistoryEvent::Hit(HistoryHit {
                actor_entity_id: 1,
                source_entity_id: Some(1),
                target_entity_id: 2,
                skill_id: 100,
                base_skill_id: Some(100),
                metric: HistoryMetric::Damage,
                amount: amount.into(),
                effective_amount: amount.into(),
                has_loss_breakdown: true,
                hp_loss: amount.into(),
                shield_loss: 0,
                flags: 0,
                target_is_boss: true,
                target_monster_id: Some(99),
                source_monster_id: None,
                property: None,
                damage_mode: None,
            }),
        }
    }

    fn skill_cast(sequence: u64) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms: 0,
            event: HistoryEvent::SkillCast(HistorySkillCast {
                caster_entity_id: 1,
                skill_id: 100,
                kind: HistoryCastKind::KeySkill,
            }),
        }
    }

    fn entity_context(sequence: u64, name_bytes: usize) -> HistoryEnvelope {
        HistoryEnvelope {
            sequence,
            offset_ms: 0,
            event: HistoryEvent::EntityContext(HistoryEntityContext {
                entity_id: i64::try_from(sequence).expect("test sequence fits i64"),
                display_uid: i64::try_from(sequence).expect("test sequence fits i64"),
                name: Some("x".repeat(name_bytes)),
                class_id: None,
                class_spec: None,
                ability_score: None,
                season_strength: None,
                monster_id: None,
            }),
        }
    }

    #[test]
    fn event_count_threshold_seals_at_8192_events() {
        let mut chunker = HistoryChunker::new(1).expect("create chunker");
        let mut chunks = Vec::new();
        for sequence in 0..=MAX_EVENTS_PER_CHUNK as u64 {
            if let Some(chunk) = chunker
                .push(skill_cast(sequence))
                .expect("append skill cast")
            {
                chunks.push(chunk);
            }
        }
        chunks.extend(chunker.flush().expect("flush chunks"));

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].stream_kind, HistoryStream::Timeline);
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[0].event_count, MAX_EVENTS_PER_CHUNK as u64);
        assert_eq!(chunks[1].chunk_index, 1);
        assert_eq!(chunks[1].event_count, 1);
    }

    #[test]
    fn hit_byte_estimate_bounds_wide_messagepack_document() {
        let mut chunker = HistoryChunker::new(1).expect("create chunker");
        let chunk = (0..MAX_EVENTS_PER_CHUNK as u64)
            .find_map(|sequence| {
                let mut envelope = hit(sequence, 0, 1);
                let HistoryEvent::Hit(hit) = &mut envelope.event else {
                    unreachable!("test helper creates hits");
                };
                hit.source_entity_id = Some(i64::MAX);
                hit.target_entity_id = i64::MAX;
                hit.skill_id = i64::MAX;
                hit.amount = u128::from(u64::MAX);
                hit.effective_amount = u128::from(u64::MAX);
                hit.hp_loss = u128::from(u64::MAX);
                hit.shield_loss = u128::from(u64::MAX);
                hit.target_monster_id = Some(i32::MAX);
                hit.source_monster_id = Some(i32::MAX);
                hit.property = Some(i32::MAX);
                hit.damage_mode = Some(i32::MAX);
                chunker.push(envelope).expect("append hit")
            })
            .expect("byte threshold seals hit chunk");
        let uncompressed = zstd::decode_all(&chunk.data[..]).expect("decompress hit chunk");

        assert!(chunk.event_count < MAX_EVENTS_PER_CHUNK as u64);
        assert!(uncompressed.len() <= TARGET_UNCOMPRESSED_CHUNK_BYTES);
    }

    #[test]
    fn estimated_byte_threshold_keeps_chunks_around_one_mib() {
        const NAME_BYTES: usize = 100_000;
        const EVENT_COUNT: u64 = 11;
        let mut chunker = HistoryChunker::new(1).expect("create chunker");
        let mut chunks = Vec::new();
        for sequence in 0..EVENT_COUNT {
            if let Some(chunk) = chunker
                .push(entity_context(sequence, NAME_BYTES))
                .expect("append entity context")
            {
                chunks.push(chunk);
            }
        }
        chunks.extend(chunker.flush().expect("flush chunks"));

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].stream_kind, HistoryStream::Context);
        assert_eq!(chunks[0].event_count, 10);
        assert_eq!(chunks[1].event_count, 1);
        assert_eq!(
            chunks.iter().map(|chunk| chunk.event_count).sum::<u64>(),
            EVENT_COUNT
        );

        let first_uncompressed =
            zstd::decode_all(&chunks[0].data[..]).expect("decompress first chunk");
        assert!(first_uncompressed.len() <= TARGET_UNCOMPRESSED_CHUNK_BYTES);
        assert!(first_uncompressed.len() >= TARGET_UNCOMPRESSED_CHUNK_BYTES * 9 / 10);
    }

    #[test]
    fn time_threshold_seals_before_two_second_span() {
        let mut chunker = HistoryChunker::new(1).expect("create chunker");
        assert!(chunker.push(hit(0, 0, 1)).expect("first hit").is_none());
        assert!(
            chunker
                .push(hit(1, MAX_CHUNK_SPAN_MS - 1, 1))
                .expect("hit inside span")
                .is_none()
        );
        let sealed = chunker
            .push(hit(2, MAX_CHUNK_SPAN_MS, 1))
            .expect("hit at span boundary")
            .expect("two second boundary seals previous chunk");
        let trailing = chunker.flush().expect("flush chunks");

        assert_eq!(sealed.event_count, 2);
        assert_eq!(sealed.start_offset_ms, 0);
        assert_eq!(sealed.end_offset_ms_exclusive, MAX_CHUNK_SPAN_MS);
        assert_eq!(trailing.len(), 1);
        assert_eq!(trailing[0].event_count, 1);
        assert_eq!(trailing[0].start_offset_ms, MAX_CHUNK_SPAN_MS);
    }

    #[test]
    fn corrupt_compressed_data_is_rejected() {
        let chunk = encode_history_chunk(1, HistoryStream::Combat, 0, vec![hit(0, 10, 1)])
            .expect("encode chunk");
        assert_eq!(
            decode_history_chunk(&chunk.data, chunk.stream_kind)
                .expect("decode chunk")
                .len(),
            1
        );

        let mut corrupted = chunk.clone();
        corrupted.data[0] ^= 0xff;
        assert!(decode_history_chunk(&corrupted.data, corrupted.stream_kind).is_err());
    }

    #[test]
    fn single_event_chunk_uses_inclusive_sequence_bounds() {
        let chunk = encode_history_chunk(1, HistoryStream::Combat, 0, vec![hit(7, 10, 1)])
            .expect("encode chunk");
        assert_eq!(chunk.first_sequence, 7);
        assert_eq!(chunk.last_sequence, 7);
        assert_eq!(chunk.event_count, 1);
    }
}
