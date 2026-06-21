use super::super::scene::SceneConfig;

const SCENE_IDS: &[i32] = &[6513, 6514, 6515];

const MECHANIC_BUFF_IDS: &[i32] = &[
    884101, 884102, 884103, 884104, 884106, 884122, 884129, 884141, 884162, 884163, 884166, 884168,
];

const RELEVANT_MONSTER_IDS: &[i32] = &[33901, 33904, 33905, 33908, 33909, 33921, 33922];

pub(crate) const CONFIG: SceneConfig = SceneConfig {
    scene_ids: SCENE_IDS,
    mechanic_buff_ids: MECHANIC_BUFF_IDS,
    relevant_monster_ids: RELEVANT_MONSTER_IDS,
};
