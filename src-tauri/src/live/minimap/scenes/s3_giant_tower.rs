use super::super::scene::SceneConfig;

const SCENE_IDS: &[i32] = &[1150, 1151, 1152];

const MECHANIC_BUFF_IDS: &[i32] = &[821076];

const RELEVANT_MONSTER_IDS: &[i32] = &[2106, 2107, 1150, 1151, 1152];

pub(crate) const CONFIG: SceneConfig = SceneConfig {
    scene_ids: SCENE_IDS,
    mechanic_buff_ids: MECHANIC_BUFF_IDS,
    relevant_monster_ids: RELEVANT_MONSTER_IDS,
};
