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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_recipe(id: &str, by: &str, inputs: Vec<(&str, u32)>, is_source: bool) -> Recipe {
        Recipe::new_for_test(
            id.to_string(),
            by.to_string(),
            60,
            inputs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            vec![(id.to_string(), 1)]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            is_source,
        )
    }

    fn create_machine(id: &str, tier: u32, power: u32) -> Machine {
        Machine {
            id: id.to_string(),
            tier,
            power,
        }
    }

    fn setup_recipes_by_output(
        item_id: &str,
        recipe_ids: Vec<&str>,
    ) -> HashMap<String, Vec<String>> {
        let mut map = HashMap::new();
        map.insert(
            item_id.to_string(),
            recipe_ids.iter().map(|s| s.to_string()).collect(),
        );
        map
    }

    #[test]
    fn test_avoids_cyclic_inputs() {
        // origocrust can be made from originium_ore or from origocrust_powder
        let recipe_cyclic =
            create_recipe("origocrust", "refining_unit", vec![("origocrust_powder", 1)], false);
        let recipe_acyclic =
            create_recipe("origocrust", "refining_unit", vec![("originium_ore", 1)], false);

        let mut recipes = HashMap::new();
        recipes.insert("recipe_cyclic".to_string(), recipe_cyclic);
        recipes.insert("recipe_acyclic".to_string(), recipe_acyclic);

        let recipes_by_output =
            setup_recipes_by_output("origocrust", vec!["recipe_cyclic", "recipe_acyclic"]);

        let mut machines = HashMap::new();
        machines.insert("refining_unit".to_string(), create_machine("refining_unit", 1, 5));

        let mut visiting = HashSet::new();
        visiting.insert("origocrust_powder".to_string());

        let selected = select_best_recipe(
            "origocrust",
            &recipes,
            &recipes_by_output,
            &machines,
            &visiting,
        );

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "origocrust");
        assert_eq!(selected.unwrap().by, "refining_unit");
        // Should select the recipe with originium_ore input to avoid cycle
        assert!(selected.unwrap().inputs.contains_key("originium_ore"));
    }

    #[test]
    fn test_prefers_is_source() {
        // buckflower_seed can be picked (is_source=true) or could hypothetically be crafted
        let recipe_source =
            create_recipe("buckflower_seed", "seed_picking_unit", vec![("buckflower", 1)], true);
        let recipe_regular = create_recipe(
            "buckflower_seed",
            "gearing_unit",
            vec![("buckflower", 2)],
            false,
        );

        let mut recipes = HashMap::new();
        recipes.insert("recipe_source".to_string(), recipe_source);
        recipes.insert("recipe_regular".to_string(), recipe_regular);

        let recipes_by_output = setup_recipes_by_output(
            "buckflower_seed",
            vec!["recipe_source", "recipe_regular"],
        );

        let mut machines = HashMap::new();
        machines.insert("seed_picking_unit".to_string(), create_machine("seed_picking_unit", 3, 10));
        machines.insert("gearing_unit".to_string(), create_machine("gearing_unit", 1, 10));

        let visiting = HashSet::new();

        let selected = select_best_recipe(
            "buckflower_seed",
            &recipes,
            &recipes_by_output,
            &machines,
            &visiting,
        );

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().by, "seed_picking_unit");
        assert!(selected.unwrap().is_source);
    }

    #[test]
    fn test_prefers_higher_tier() {
        // originium_ore can be mined by different tier machines
        let recipe_tier1 = create_recipe("originium_ore", "portable_originium_rig", vec![], true);
        let recipe_tier2 = create_recipe("originium_ore", "electric_mining_rig", vec![], true);
        let recipe_tier3 = create_recipe("originium_ore", "electric_mining_rig_mk2", vec![], true);

        let mut recipes = HashMap::new();
        recipes.insert("recipe_tier1".to_string(), recipe_tier1);
        recipes.insert("recipe_tier2".to_string(), recipe_tier2);
        recipes.insert("recipe_tier3".to_string(), recipe_tier3);

        let recipes_by_output = setup_recipes_by_output(
            "originium_ore",
            vec!["recipe_tier1", "recipe_tier2", "recipe_tier3"],
        );

        let mut machines = HashMap::new();
        machines.insert(
            "portable_originium_rig".to_string(),
            create_machine("portable_originium_rig", 1, 0),
        );
        machines.insert(
            "electric_mining_rig".to_string(),
            create_machine("electric_mining_rig", 2, 5),
        );
        machines.insert(
            "electric_mining_rig_mk2".to_string(),
            create_machine("electric_mining_rig_mk2", 3, 10),
        );

        let visiting = HashSet::new();

        let selected = select_best_recipe(
            "originium_ore",
            &recipes,
            &recipes_by_output,
            &machines,
            &visiting,
        );

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().by, "electric_mining_rig_mk2");
    }

    #[test]
    fn test_prefers_lower_power() {
        // At same tier, prefer lower power consumption
        // Both tier 2, but different power
        let recipe_high_power =
            create_recipe("amethyst_ore", "electric_mining_rig", vec![], true);
        let recipe_low_power = create_recipe("amethyst_ore", "fluid_pump", vec![], true);

        let mut recipes = HashMap::new();
        recipes.insert("recipe_high_power".to_string(), recipe_high_power);
        recipes.insert("recipe_low_power".to_string(), recipe_low_power);

        let recipes_by_output = setup_recipes_by_output(
            "amethyst_ore",
            vec!["recipe_high_power", "recipe_low_power"],
        );

        let mut machines = HashMap::new();
        // Same tier, different power
        machines.insert(
            "electric_mining_rig".to_string(),
            create_machine("electric_mining_rig", 2, 10),
        );
        machines.insert("fluid_pump".to_string(), create_machine("fluid_pump", 2, 5));

        let visiting = HashSet::new();

        let selected = select_best_recipe(
            "amethyst_ore",
            &recipes,
            &recipes_by_output,
            &machines,
            &visiting,
        );

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().by, "fluid_pump");
    }

    #[test]
    fn test_returns_none_when_no_candidates() {
        let recipes = HashMap::new();
        let recipes_by_output = HashMap::new();
        let machines = HashMap::new();
        let visiting = HashSet::new();

        let selected = select_best_recipe(
            "nonexistent_item",
            &recipes,
            &recipes_by_output,
            &machines,
            &visiting,
        );

        assert!(selected.is_none());
    }
}
