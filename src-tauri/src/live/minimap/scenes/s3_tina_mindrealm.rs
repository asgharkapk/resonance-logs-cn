use super::super::scene::SceneConfig;

const SCENE_IDS: &[i32] = &[1631, 1632, 1633];

const MECHANIC_BUFF_IDS: &[i32] = &[510571, 841519, 841509];

const RELEVANT_MONSTER_IDS: &[i32] = &[33701, 300086, 300089];

pub(crate) const CONFIG: SceneConfig = SceneConfig {
    scene_ids: SCENE_IDS,
    mechanic_buff_ids: MECHANIC_BUFF_IDS,
    relevant_monster_ids: RELEVANT_MONSTER_IDS,
};
