use crate::live::counter::engine::{CounterRule, CounterSource, EffectSlotConfig};
use std::collections::HashSet;

const FACTOR_RULE_ID_BASE: i32 = 900_000_000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FactorCounterTemplate {
    #[serde(default)]
    pub item_ids: Vec<i32>,
    #[serde(default)]
    pub sources: Vec<CounterSource>,
    #[serde(default)]
    pub effect_slots: Vec<EffectSlotConfig>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonCultivateFactorSelection {
    pub source_item_ids: Vec<i32>,
    pub slot_item_ids: Vec<i32>,
}

/// Protocol-neutral factor rule compiler. Its input is already decoded and
/// normalized by the protocol boundary.
#[derive(Debug, Clone, Default)]
pub struct FactorCounterCompiler {
    templates: Vec<FactorCounterTemplate>,
    active_item_ids: Vec<i32>,
    active_signature: String,
    active_selection: SeasonCultivateFactorSelection,
}

impl FactorCounterCompiler {
    pub fn set_templates(
        &mut self,
        templates: Vec<FactorCounterTemplate>,
    ) -> Option<Vec<CounterRule>> {
        self.templates = normalize_factor_templates(templates);
        self.active_signature.clear();
        self.rebuild()
    }

    pub fn set_active_item_ids(&mut self, active_item_ids: Vec<i32>) -> Option<Vec<CounterRule>> {
        self.active_item_ids = normalized_item_ids(active_item_ids);
        self.rebuild()
    }

    #[must_use]
    pub fn active_selection(&self) -> SeasonCultivateFactorSelection {
        self.active_selection.clone()
    }

    fn rebuild(&mut self) -> Option<Vec<CounterRule>> {
        let selection = select_factor_items(&self.templates, &self.active_item_ids);
        let signature = build_selection_signature(&selection);
        if signature == self.active_signature {
            return None;
        }
        self.active_signature = signature;
        self.active_selection = selection.clone();
        Some(build_counter_rules(&self.templates, &selection))
    }
}

fn normalized_item_ids(mut item_ids: Vec<i32>) -> Vec<i32> {
    item_ids.retain(|item_id| *item_id > 0);
    item_ids.sort_unstable();
    item_ids.dedup();
    item_ids
}

fn select_factor_items(
    templates: &[FactorCounterTemplate],
    active_item_ids: &[i32],
) -> SeasonCultivateFactorSelection {
    let source_ids = template_item_id_set(
        templates
            .iter()
            .filter(|template| !template.sources.is_empty()),
    );
    let slot_ids = template_item_id_set(
        templates
            .iter()
            .filter(|template| !template.effect_slots.is_empty()),
    );
    let source_item_ids = normalized_item_ids(
        active_item_ids
            .iter()
            .copied()
            .filter(|item_id| source_ids.contains(item_id))
            .collect(),
    );
    let slot_item_ids = normalized_item_ids(
        active_item_ids
            .iter()
            .copied()
            .filter(|item_id| slot_ids.contains(item_id))
            .collect(),
    );
    SeasonCultivateFactorSelection {
        source_item_ids,
        slot_item_ids,
    }
}

fn build_counter_rules(
    templates: &[FactorCounterTemplate],
    selection: &SeasonCultivateFactorSelection,
) -> Vec<CounterRule> {
    if selection.source_item_ids.is_empty() || selection.slot_item_ids.is_empty() {
        return Vec::new();
    }
    let source_templates: Vec<&FactorCounterTemplate> = templates
        .iter()
        .filter(|template| {
            !template.sources.is_empty()
                && template_matches_any_item_id(template, &selection.source_item_ids)
        })
        .collect();
    if source_templates.is_empty() {
        return Vec::new();
    }
    let sources: Vec<CounterSource> = source_templates
        .iter()
        .flat_map(|template| template.sources.iter().cloned())
        .collect();
    selection
        .slot_item_ids
        .iter()
        .filter_map(|slot_item_id| {
            let template = templates.iter().find(|template| {
                !template.effect_slots.is_empty()
                    && template_matches_item_id(template, *slot_item_id)
            })?;
            Some(CounterRule {
                rule_id: factor_rule_id(*slot_item_id),
                sources: sources.clone(),
                effect_slots: template
                    .effect_slots
                    .iter()
                    .enumerate()
                    .map(|(index, slot)| {
                        let mut next = slot.clone();
                        next.slot_id = i32::try_from(index + 1).unwrap_or(i32::MAX);
                        next
                    })
                    .collect(),
            })
        })
        .collect()
}

pub fn factor_rule_id(item_id: i32) -> i32 {
    FACTOR_RULE_ID_BASE.saturating_add(item_id)
}

fn build_selection_signature(selection: &SeasonCultivateFactorSelection) -> String {
    format!(
        "{:?}|{:?}",
        selection.source_item_ids, selection.slot_item_ids
    )
}

fn template_item_id_set<'a>(
    templates: impl Iterator<Item = &'a FactorCounterTemplate>,
) -> HashSet<i32> {
    let mut result = HashSet::new();
    for template in templates {
        result.extend(template.item_ids.iter().copied());
    }
    result
}

fn template_matches_any_item_id(template: &FactorCounterTemplate, item_ids: &[i32]) -> bool {
    item_ids
        .iter()
        .any(|item_id| template_matches_item_id(template, *item_id))
}

fn template_matches_item_id(template: &FactorCounterTemplate, item_id: i32) -> bool {
    template.item_ids.contains(&item_id)
}

pub fn normalize_factor_templates(
    templates: Vec<FactorCounterTemplate>,
) -> Vec<FactorCounterTemplate> {
    templates
        .into_iter()
        .filter_map(|mut template| {
            template.item_ids.retain(|item_id| *item_id > 0);
            template.item_ids.sort_unstable();
            template.item_ids.dedup();
            (!template.item_ids.is_empty()).then_some(template)
        })
        .collect()
}
