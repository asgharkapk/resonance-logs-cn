use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MonsterType {
    Normal = 0,
    Elite = 1,
    Boss = 2,
}

static MONSTER_REGISTRY: LazyLock<HashMap<i32, MonsterType>> = LazyLock::new(|| {
    let data = include_str!("../../meter-data/MonsterIdType.json");
    let raw: HashMap<String, u8> = serde_json::from_str(data).expect("invalid MonsterIdType.json");

    let mut registry = HashMap::with_capacity(raw.len());
    for (key, monster_type) in raw {
        if let Ok(id) = key.parse::<i32>() {
            let monster_type = match monster_type {
                1 => MonsterType::Elite,
                2 => MonsterType::Boss,
                _ => MonsterType::Normal,
            };

            registry.insert(id, monster_type);
        }
    }

    registry
});

pub fn monster_type(id: i32) -> Option<MonsterType> {
    MONSTER_REGISTRY.get(&id).copied()
}
