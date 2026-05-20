use crate::live::opcodes_models::{Encounter, attr_type};
use blueprotobuf_lib::blueprotobuf::{AoiSyncDelta, EDamageType};
use std::time::{Duration, Instant};

pub const TRAINING_SEGMENT_DURATION: Duration = Duration::from_secs(183);

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, specta::Type,
)]
#[serde(rename_all = "camelCase")]
pub enum TrainingDummyPhase {
    #[default]
    Idle,
    Armed,
    Running,
    PendingRollover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TrainingDummyMonsterId {
    EliteEnemy = 115,
    EliteGuardian = 122,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("unsupported training dummy monster id: {0}")]
pub struct InvalidTrainingDummyMonsterId(pub i32);

impl TrainingDummyMonsterId {
    pub fn id(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for TrainingDummyMonsterId {
    type Error = InvalidTrainingDummyMonsterId;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            115 => Ok(Self::EliteEnemy),
            122 => Ok(Self::EliteGuardian),
            _ => Err(InvalidTrainingDummyMonsterId(value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrainingDummyMatch {
    pub target_entity_uuid: i64,
    pub monster_id: TrainingDummyMonsterId,
    pub has_local_player_damage: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TrainingDummyRuntime {
    pub phase: TrainingDummyPhase,
    pub selected_monster_id: Option<TrainingDummyMonsterId>,
    pub locked_target_uuid: Option<i64>,
    pub rollover_ready_at: Option<Instant>,
}

impl TrainingDummyRuntime {
    pub fn arm(&mut self, monster_id: TrainingDummyMonsterId) {
        self.phase = TrainingDummyPhase::Armed;
        self.selected_monster_id = Some(monster_id);
        self.locked_target_uuid = None;
        self.rollover_ready_at = None;
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn has_selection(&self) -> bool {
        self.selected_monster_id.is_some()
    }

    pub fn rearm_selected(&mut self) {
        if let Some(monster_id) = self.selected_monster_id {
            self.arm(monster_id);
        } else {
            self.clear();
        }
    }

    pub fn combat_target_filter(&self) -> Option<i64> {
        match self.phase {
            TrainingDummyPhase::Running | TrainingDummyPhase::PendingRollover => {
                self.locked_target_uuid
            }
            TrainingDummyPhase::Idle | TrainingDummyPhase::Armed => None,
        }
    }

    pub fn maybe_enter_pending_rollover(&mut self) {
        if self.phase != TrainingDummyPhase::Running {
            return;
        }
        if self
            .rollover_ready_at
            .is_some_and(|trigger_at| Instant::now() >= trigger_at)
        {
            self.phase = TrainingDummyPhase::PendingRollover;
        }
    }

    pub fn should_lock_on_match(&self, matched: TrainingDummyMatch) -> bool {
        self.phase == TrainingDummyPhase::Armed
            && self.selected_monster_id == Some(matched.monster_id)
            && matched.has_local_player_damage
    }

    pub fn should_rollover_on_match(&self, matched: TrainingDummyMatch) -> bool {
        self.phase == TrainingDummyPhase::PendingRollover
            && self.locked_target_uuid == Some(matched.target_entity_uuid)
            && matched.has_local_player_damage
    }

    pub fn lock_target(&mut self, matched: TrainingDummyMatch) {
        let now = Instant::now();
        self.phase = TrainingDummyPhase::Running;
        self.selected_monster_id = Some(matched.monster_id);
        self.locked_target_uuid = Some(matched.target_entity_uuid);
        self.rollover_ready_at = Some(now + TRAINING_SEGMENT_DURATION);
    }
}

pub fn inspect_aoi_delta(
    encounter: &Encounter,
    delta: &AoiSyncDelta,
    local_player_uuid: i64,
) -> Option<TrainingDummyMatch> {
    let target_uuid = delta.uuid?;
    let monster_id = resolve_target_monster_id(encounter, delta, target_uuid)?;
    let has_local_player_damage = delta.skill_effects.as_ref().is_some_and(|effects| {
        effects
            .damages
            .iter()
            .any(|damage| is_local_player_damage(damage, local_player_uuid))
    });

    Some(TrainingDummyMatch {
        target_entity_uuid: target_uuid,
        monster_id,
        has_local_player_damage,
    })
}

fn resolve_target_monster_id(
    encounter: &Encounter,
    delta: &AoiSyncDelta,
    target_uuid: i64,
) -> Option<TrainingDummyMonsterId> {
    let attrs_monster_id = delta.attrs.as_ref().and_then(|attrs| {
        attrs.attrs.iter().find_map(|attr| {
            (attr.id == Some(attr_type::ATTR_ID))
                .then(|| {
                    attr.raw_data
                        .as_deref()
                        .and_then(|raw| decode_attr_id(Some(raw)))
                })
                .flatten()
        })
    });

    attrs_monster_id
        .or_else(|| {
            encounter
                .entity_uuid_to_entity
                .get(&target_uuid)
                .and_then(|entity| entity.monster_type_id)
        })
        .and_then(|monster_id| TrainingDummyMonsterId::try_from(monster_id).ok())
}

fn decode_attr_id(raw: Option<&[u8]>) -> Option<i32> {
    let mut buf = raw?;
    prost::encoding::decode_varint(&mut buf)
        .ok()
        .and_then(|value| i32::try_from(value).ok())
}

fn is_local_player_damage(
    damage: &blueprotobuf_lib::blueprotobuf::SyncDamageInfo,
    local_player_uuid: i64,
) -> bool {
    if local_player_uuid <= 0 {
        return false;
    }
    if damage.r#type.unwrap_or(0) == EDamageType::Heal as i32 {
        return false;
    }
    if damage.value.is_none() && damage.lucky_value.is_none() {
        return false;
    }
    if damage.owner_id.is_none() {
        return false;
    }

    damage
        .top_summoner_id
        .or(damage.attacker_uuid)
        .map(|uuid| uuid == local_player_uuid)
        .unwrap_or(false)
}
