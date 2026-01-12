//! Recipe selection logic for production planning.

use crate::models::{Machine, Recipe};
use std::collections::{HashMap, HashSet};

fn has_cyclic_inputs(recipe: &Recipe, visiting: &HashSet<String>) -> bool {
    recipe
        .inputs
        .keys()
        .any(|input_id| visiting.contains(input_id))
}

/// Selects the best recipe for a given item based on priority rules.
///
/// Priority (highest to lowest):
/// 1. Higher machine tier
/// 2. Lower power consumption
/// 3. Alphabetical recipe ID (for determinism)
///
/// Returns `None` if no recipe exists for the item.
pub fn select_best_recipe<'a>(
    item_id: &str,
    recipes: &'a HashMap<String, Recipe>,
    recipes_by_output: &HashMap<String, Vec<String>>,
    machines: &HashMap<String, Machine>,
    visiting: &HashSet<String>,
) -> Option<&'a Recipe> {
    recipes_by_output.get(item_id).and_then(|candidates| {
        candidates
            .iter()
            .filter_map(|id| recipes.get(id))
            .max_by(|recipe_a, recipe_b| {
                let machine_a = machines.get(&recipe_a.by);
                let machine_b = machines.get(&recipe_b.by);

                let tier_a = machine_a.map(|m| m.tier).unwrap_or(0);
                let tier_b = machine_b.map(|m| m.tier).unwrap_or(0);

                let power_a = machine_a.map(|m| m.power).unwrap_or(0);
                let power_b = machine_b.map(|m| m.power).unwrap_or(0);

                let cyclic_a = has_cyclic_inputs(recipe_a, visiting);
                let cyclic_b = has_cyclic_inputs(recipe_b, visiting);

                cyclic_b
                    .cmp(&cyclic_a)
                    .then_with(|| recipe_a.is_source.cmp(&recipe_b.is_source))
                    .then_with(|| tier_a.cmp(&tier_b))
                    .then_with(|| power_b.cmp(&power_a))
                    .then_with(|| recipe_a.id.cmp(&recipe_b.id))
            })
    })
}
