//! Skill cooldown reduction / accelerate calculation.

use log::warn;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::LazyLock;

use crate::live::damage_id::locate_meter_data_file;

const TEMP_ATTR_TABLE_RELATIVE: &str = "meter-data/TempAttrTable.json";
const SKILL_EFFECT_TABLE_RELATIVE: &str = "meter-data/SkillEffectTable.json";
const TAG_NO_CD_REDUCE: i32 = 103;

#[derive(Debug, Clone, Deserialize)]
struct RawTempAttrDef {
    #[serde(rename = "Id")]
    id: i32,
    #[serde(rename = "AttrType")]
    attr_type: i32,
    #[serde(rename = "LogicType")]
    logic_type: i32,
    #[serde(rename = "AttrParams", default)]
    attr_params: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct CdTempAttrDef {
    pub attr_type: i32,
    pub logic_type: i32,
    pub attr_params: Vec<i32>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawSkillEffectEntry {
    #[serde(rename = "Tags", default)]
    tags: Vec<i32>,
}

static CD_TEMP_ATTR_DEFS: LazyLock<HashMap<i32, CdTempAttrDef>> = LazyLock::new(|| {
    load_cd_temp_attr_defs().unwrap_or_else(|err| {
        warn!("[skill-cd] failed to load TempAttrTable.json: {}", err);
        HashMap::new()
    })
});

static SKILL_EFFECT_TAGS: LazyLock<HashMap<i32, Vec<i32>>> = LazyLock::new(|| {
    load_skill_effect_tags().unwrap_or_else(|err| {
        warn!("[skill-cd] failed to load SkillEffectTable.json: {}", err);
        HashMap::new()
    })
});

/// Apply AttrSkillCD / AttrSkillCDPCT / AttrCDAcceleratePCT and matching TempAttr
/// rules to a base cooldown duration.
///
/// Returns `(calculated_duration_ms, accelerate_rate)`.
#[must_use]
pub fn calculate_skill_cd(
    base_cd: f32,
    skill_level_id: i32,
    temp_attr_values: &HashMap<i32, i32>,
    attr_skill_cd: f32,
    attr_skill_cd_pct: f32,
    attr_cd_accelerate_pct: f32,
) -> (f32, f32) {
    calculate_skill_cd_with_tables(
        base_cd,
        skill_level_id,
        temp_attr_values,
        attr_skill_cd,
        attr_skill_cd_pct,
        attr_cd_accelerate_pct,
        &CD_TEMP_ATTR_DEFS,
        &SKILL_EFFECT_TAGS,
    )
}

/// Pure variant that accepts injected table references (for unit tests).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn calculate_skill_cd_with_tables(
    base_cd: f32,
    skill_level_id: i32,
    temp_attr_values: &HashMap<i32, i32>,
    attr_skill_cd: f32,
    attr_skill_cd_pct: f32,
    attr_cd_accelerate_pct: f32,
    temp_attr_defs: &HashMap<i32, CdTempAttrDef>,
    skill_effect_tags: &HashMap<i32, Vec<i32>>,
) -> (f32, f32) {
    if base_cd <= 0.0 {
        return (0.0, 0.0);
    }

    let skill_id = skill_level_id / 100;
    let tag_lookup_skill_level_id = skill_id * 100 + 1;
    let skill_tags_vec = skill_effect_tags
        .get(&tag_lookup_skill_level_id)
        .cloned()
        .unwrap_or_default();
    let skill_tags: HashSet<i32> = skill_tags_vec.iter().copied().collect();

    if skill_tags.contains(&TAG_NO_CD_REDUCE) {
        return (base_cd.max(0.0), 0.0);
    }

    let mut flat_reduce = attr_skill_cd;
    let mut pct_reduce = attr_skill_cd_pct / 10000.0;
    let mut accelerate = attr_cd_accelerate_pct / 10000.0;

    for (temp_attr_id, value) in temp_attr_values {
        if *value == 0 {
            continue;
        }
        let Some(def) = temp_attr_defs.get(temp_attr_id) else {
            continue;
        };
        if !temp_attr_matches(def, skill_id, &skill_tags) {
            continue;
        }

        match def.attr_type {
            101 => {
                flat_reduce += *value as f32 / 1000.0;
            }
            100 => {
                pct_reduce += *value as f32 / 10000.0;
            }
            103 => {
                accelerate += *value as f32 / 10000.0;
            }
            _ => {}
        }
    }

    let reduced_cd = ((1.0 - pct_reduce) * (base_cd - flat_reduce)).max(0.0);
    (reduced_cd, accelerate)
}

fn temp_attr_matches(def: &CdTempAttrDef, skill_id: i32, skill_tags: &HashSet<i32>) -> bool {
    match def.logic_type {
        0 => true,
        1 => def.attr_params.contains(&skill_id),
        3 => def.attr_params.iter().any(|tag| skill_tags.contains(tag)),
        _ => false,
    }
}

fn load_cd_temp_attr_defs() -> Result<HashMap<i32, CdTempAttrDef>, Box<dyn std::error::Error>> {
    let path = locate_meter_data_file(TEMP_ATTR_TABLE_RELATIVE)
        .ok_or_else(|| format!("{TEMP_ATTR_TABLE_RELATIVE} not found in known locations"))?;
    let contents = fs::read_to_string(path)?;
    let raw_map: HashMap<String, RawTempAttrDef> = serde_json::from_str(&contents)?;

    let mut result = HashMap::new();
    for raw in raw_map.into_values() {
        // 100 = pct reduce, 101 = flat reduce, 103 = accelerate pct
        if raw.attr_type != 100 && raw.attr_type != 101 && raw.attr_type != 103 {
            continue;
        }
        result.insert(
            raw.id,
            CdTempAttrDef {
                attr_type: raw.attr_type,
                logic_type: raw.logic_type,
                attr_params: raw.attr_params,
            },
        );
    }
    Ok(result)
}

fn load_skill_effect_tags() -> Result<HashMap<i32, Vec<i32>>, Box<dyn std::error::Error>> {
    let path = locate_meter_data_file(SKILL_EFFECT_TABLE_RELATIVE)
        .ok_or_else(|| format!("{SKILL_EFFECT_TABLE_RELATIVE} not found in known locations"))?;
    let contents = fs::read_to_string(path)?;
    let raw_map: HashMap<String, RawSkillEffectEntry> = serde_json::from_str(&contents)?;

    let mut result = HashMap::new();
    for (key, value) in raw_map {
        if let Ok(skill_level_id) = key.parse::<i32>() {
            result.insert(skill_level_id, value.tags);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_tables() -> (HashMap<i32, CdTempAttrDef>, HashMap<i32, Vec<i32>>) {
        (HashMap::new(), HashMap::new())
    }

    #[test]
    fn non_positive_base_cd_returns_zeros() {
        let (defs, tags) = empty_tables();
        let (duration, accelerate) = calculate_skill_cd_with_tables(
            0.0,
            12_301,
            &HashMap::new(),
            100.0,
            1_000.0,
            500.0,
            &defs,
            &tags,
        );
        assert_eq!(duration, 0.0);
        assert_eq!(accelerate, 0.0);
    }

    #[test]
    fn panel_attrs_apply_flat_pct_and_accelerate() {
        let (defs, tags) = empty_tables();
        // flat 1000ms, pct 10% (1000/10000), accelerate 5% (500/10000)
        let (duration, accelerate) = calculate_skill_cd_with_tables(
            10_000.0,
            12_301,
            &HashMap::new(),
            1_000.0,
            1_000.0,
            500.0,
            &defs,
            &tags,
        );
        // (1 - 0.1) * (10000 - 1000) = 8100
        assert!((duration - 8_100.0).abs() < f32::EPSILON);
        assert!((accelerate - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn tag_no_cd_reduce_skips_all_reductions() {
        let (defs, mut tags) = empty_tables();
        tags.insert(12_301, vec![TAG_NO_CD_REDUCE]);
        let (duration, accelerate) = calculate_skill_cd_with_tables(
            10_000.0,
            12_301,
            &HashMap::new(),
            1_000.0,
            1_000.0,
            500.0,
            &defs,
            &tags,
        );
        assert!((duration - 10_000.0).abs() < f32::EPSILON);
        assert_eq!(accelerate, 0.0);
    }

    #[test]
    fn logic_type_0_temp_attr_applies_to_all_skills() {
        let (mut defs, tags) = empty_tables();
        defs.insert(
            77,
            CdTempAttrDef {
                attr_type: 100, // pct
                logic_type: 0,
                attr_params: Vec::new(),
            },
        );
        let mut values = HashMap::new();
        values.insert(77, 2_000); // +20%
        let (duration, _) =
            calculate_skill_cd_with_tables(10_000.0, 12_301, &values, 0.0, 0.0, 0.0, &defs, &tags);
        // (1 - 0.2) * 10000 = 8000
        assert!((duration - 8_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn logic_type_1_temp_attr_matches_skill_id_only() {
        let (mut defs, tags) = empty_tables();
        defs.insert(
            88,
            CdTempAttrDef {
                attr_type: 101, // flat ms (value / 1000)
                logic_type: 1,
                attr_params: vec![123],
            },
        );
        let mut values = HashMap::new();
        values.insert(88, 2_000_000); // 2_000_000/1000 = 2000ms flat

        let (matched, _) = calculate_skill_cd_with_tables(
            10_000.0, 12_301, // skill_id = 123
            &values, 0.0, 0.0, 0.0, &defs, &tags,
        );
        assert!((matched - 8_000.0).abs() < f32::EPSILON);

        let (unmatched, _) = calculate_skill_cd_with_tables(
            10_000.0, 45_601, // skill_id = 456
            &values, 0.0, 0.0, 0.0, &defs, &tags,
        );
        assert!((unmatched - 10_000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn logic_type_3_temp_attr_matches_skill_tags() {
        let (mut defs, mut tags) = empty_tables();
        defs.insert(
            99,
            CdTempAttrDef {
                attr_type: 103, // accelerate
                logic_type: 3,
                attr_params: vec![7],
            },
        );
        tags.insert(12_301, vec![7, 9]);
        let mut values = HashMap::new();
        values.insert(99, 1_500); // 0.15

        let (_, accelerate) =
            calculate_skill_cd_with_tables(10_000.0, 12_301, &values, 0.0, 0.0, 0.0, &defs, &tags);
        assert!((accelerate - 0.15).abs() < f32::EPSILON);
    }
}
