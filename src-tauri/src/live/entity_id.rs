use blueprotobuf_lib::blueprotobuf::EEntityType;

pub type EntityUuid = i64;
pub type EntityUid = i64;

pub const ENTITY_TYPE_SHIFT: i64 = 6;
pub const ENTITY_UID_SHIFT: i64 = 16;

#[inline]
pub fn uid_from_uuid(uuid: EntityUuid) -> EntityUid {
    uuid >> ENTITY_UID_SHIFT
}

#[inline]
pub fn entity_uuid_string(uuid: EntityUuid) -> String {
    uuid.to_string()
}

#[inline]
pub fn canonical_player_uuid(char_id: EntityUid) -> EntityUuid {
    entity_id_to_uuid(char_id, EEntityType::EntChar, false, false)
}

#[inline]
pub fn entity_id_to_uuid(
    uid: EntityUid,
    entity_type: EEntityType,
    is_summon: bool,
    is_client: bool,
) -> EntityUuid {
    let summon_bit = if is_summon { 1_i64 } else { 0 };
    let client_bit = if is_client { 1_i64 } else { 0 };
    (uid << ENTITY_UID_SHIFT)
        | (summon_bit << 15)
        | (client_bit << 14)
        | (i64::from(entity_type as i32) << ENTITY_TYPE_SHIFT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_player_uuid_round_trips_to_uid() {
        let uid = 12_345_678;
        let uuid = canonical_player_uuid(uid);

        assert_eq!(uid_from_uuid(uuid), uid);
        assert_eq!(EEntityType::from(uuid), EEntityType::EntChar);
    }

    #[test]
    fn same_uid_can_have_distinct_entity_uuids() {
        let uid = 42;
        let player_uuid = entity_id_to_uuid(uid, EEntityType::EntChar, false, false);
        let monster_uuid = entity_id_to_uuid(uid, EEntityType::EntMonster, false, false);

        assert_eq!(uid_from_uuid(player_uuid), uid);
        assert_eq!(uid_from_uuid(monster_uuid), uid);
        assert_ne!(player_uuid, monster_uuid);
        assert_ne!(
            EEntityType::from(player_uuid),
            EEntityType::from(monster_uuid)
        );
    }
}
