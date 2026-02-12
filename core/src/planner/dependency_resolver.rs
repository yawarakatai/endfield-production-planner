//! Dependency resolution for production planning.

use crate::models::{Machine, ProductionNode, Recipe};
use std::collections::{HashMap, HashSet};

use super::calculator;
use super::recipe_selector;

/// Recursively resolves production dependencies for an item.
///
/// # Arguments
/// * `recipes` - All available recipes indexed by unique ID
/// * `recipes_by_output` - Recipe IDs indexed by output item
/// * `machines` - All available machines indexed by ID
/// * `item_id` - The item to produce
/// * `amount` - Desired output amount per time window
/// * `visiting` - Set of items currently being resolved (for cycle detection)
///
/// # Returns
/// A `ProductionNode` representing the production tree for the item.
pub fn resolve(
    recipes: &HashMap<String, Recipe>,
    recipes_by_output: &HashMap<String, Vec<String>>,
    machines: &HashMap<String, Machine>,
    item_id: &str,
    amount: u32,
    visiting: &mut HashSet<String>,
) -> ProductionNode {
    // Mark item as being visited (cycle detection)
    visiting.insert(item_id.to_string());

    let result = match recipe_selector::select_best_recipe(
        item_id,
        recipes,
        recipes_by_output,
        machines,
        visiting,
    ) {
        Some(recipe) => build_resolved_node(
            recipe,
            recipes,
            recipes_by_output,
            machines,
            item_id,
            amount,
            visiting,
        ),
        None => ProductionNode::Unresolved {
            item_id: item_id.to_string(),
            amount,
        },
    };

    // Backtrack
    visiting.remove(item_id);

    result
}

/// Builds a resolved production node with its children.
fn build_resolved_node(
    recipe: &Recipe,
    recipes: &HashMap<String, Recipe>,
    recipes_by_output: &HashMap<String, Vec<String>>,
    machines: &HashMap<String, Machine>,
    item_id: &str,
    amount: u32,
    visiting: &mut HashSet<String>,
) -> ProductionNode {
    let machine = machines.get(&recipe.by);
    let machine_id = machine
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "missing_machine".to_string());

    let calc = calculator::calculate(recipe, machine, amount, item_id);

    let children: Vec<ProductionNode> = recipe
        .inputs
        .iter()
        .filter_map(|(input_id, input_count)| {
            // Skip if already visiting (cycle prevention)
            if visiting.contains(input_id) {
                return None;
            }

            let sub_amount = (*input_count as f64 * calc.required_crafts).ceil() as u32;

            Some(resolve(
                recipes,
                recipes_by_output,
                machines,
                input_id,
                sub_amount,
                visiting,
            ))
        })
        .collect();

    ProductionNode::Resolved {
        item_id: item_id.to_string(),
        machine_id,
        amount,
        machine_count: calc.machine_count,
        load: calc.load,
        power_usage: calc.power_usage,
        inputs: children,
        is_source: recipe.is_source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_recipe(
        id: &str,
        by: &str,
        inputs: Vec<(&str, u32)>,
        outputs: Vec<(&str, u32)>,
    ) -> Recipe {
        Recipe::new_for_test(
            id.to_string(),
            by.to_string(),
            60,
            inputs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            outputs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            false,
        )
    }

    fn create_machine(id: &str, tier: u32, power: u32) -> Machine {
        Machine {
            id: id.to_string(),
            tier,
            power,
        }
    }

    #[test]
    fn test_linear_dependency() {
        // origocrust_powder requires originium_powder, which requires originium_ore
        let recipe_ore = create_recipe(
            "originium_ore",
            "electric_mining_rig",
            vec![],
            vec![("originium_ore", 1)],
        );
        let recipe_powder = create_recipe(
            "originium_powder",
            "shredding_unit",
            vec![("originium_ore", 1)],
            vec![("originium_powder", 1)],
        );
        let recipe_crust_powder = create_recipe(
            "origocrust_powder",
            "refining_unit",
            vec![("originium_powder", 1)],
            vec![("origocrust_powder", 1)],
        );

        let mut recipes = HashMap::new();
        recipes.insert(
            "originium_ore@electric_mining_rig[]".to_string(),
            recipe_ore,
        );
        recipes.insert(
            "originium_powder@shredding_unit[originium_ore:1]".to_string(),
            recipe_powder,
        );
        recipes.insert(
            "origocrust_powder@refining_unit[originium_powder:1]".to_string(),
            recipe_crust_powder,
        );

        let mut recipes_by_output = HashMap::new();
        recipes_by_output.insert(
            "originium_ore".to_string(),
            vec!["originium_ore@electric_mining_rig[]".to_string()],
        );
        recipes_by_output.insert(
            "originium_powder".to_string(),
            vec!["originium_powder@shredding_unit[originium_ore:1]".to_string()],
        );
        recipes_by_output.insert(
            "origocrust_powder".to_string(),
            vec!["origocrust_powder@refining_unit[originium_powder:1]".to_string()],
        );

        let mut machines = HashMap::new();
        machines.insert(
            "electric_mining_rig".to_string(),
            create_machine("electric_mining_rig", 2, 5),
        );
        machines.insert(
            "shredding_unit".to_string(),
            create_machine("shredding_unit", 1, 10),
        );
        machines.insert(
            "refining_unit".to_string(),
            create_machine("refining_unit", 1, 5),
        );

        let mut visiting = HashSet::new();
        let result = resolve(
            &recipes,
            &recipes_by_output,
            &machines,
            "origocrust_powder",
            1,
            &mut visiting,
        );

        match result {
            ProductionNode::Resolved {
                item_id, inputs, ..
            } => {
                assert_eq!(item_id, "origocrust_powder");
                assert_eq!(inputs.len(), 1);

                match &inputs[0] {
                    ProductionNode::Resolved {
                        item_id: powder_id,
                        inputs: powder_inputs,
                        ..
                    } => {
                        assert_eq!(powder_id, "originium_powder");
                        assert_eq!(powder_inputs.len(), 1);

                        match &powder_inputs[0] {
                            ProductionNode::Resolved {
                                item_id: ore_id, ..
                            } => {
                                assert_eq!(ore_id, "originium_ore");
                            }
                            _ => panic!("Expected Resolved node for originium_ore"),
                        }
                    }
                    _ => panic!("Expected Resolved node for originium_powder"),
                }
            }
            _ => panic!("Expected Resolved node for origocrust_powder"),
        }
    }

    #[test]
    fn test_branching_dependency() {
        // amethyst_component requires both amethyst_fiber and origocrust
        let recipe_fiber = create_recipe(
            "amethyst_fiber",
            "refining_unit",
            vec![],
            vec![("amethyst_fiber", 1)],
        );
        let recipe_crust = create_recipe(
            "origocrust",
            "refining_unit",
            vec![],
            vec![("origocrust", 1)],
        );
        let recipe_component = create_recipe(
            "amethyst_component",
            "gearing_unit",
            vec![("amethyst_fiber", 5), ("origocrust", 5)],
            vec![("amethyst_component", 1)],
        );

        let mut recipes = HashMap::new();
        recipes.insert("amethyst_fiber@refining_unit[]".to_string(), recipe_fiber);
        recipes.insert("origocrust@refining_unit[]".to_string(), recipe_crust);
        recipes.insert(
            "amethyst_component@gearing_unit[amethyst_fiber:5,origocrust:5]".to_string(),
            recipe_component,
        );

        let mut recipes_by_output = HashMap::new();
        recipes_by_output.insert(
            "amethyst_fiber".to_string(),
            vec!["amethyst_fiber@refining_unit[]".to_string()],
        );
        recipes_by_output.insert(
            "origocrust".to_string(),
            vec!["origocrust@refining_unit[]".to_string()],
        );
        recipes_by_output.insert(
            "amethyst_component".to_string(),
            vec!["amethyst_component@gearing_unit[amethyst_fiber:5,origocrust:5]".to_string()],
        );

        let mut machines = HashMap::new();
        machines.insert(
            "refining_unit".to_string(),
            create_machine("refining_unit", 1, 5),
        );
        machines.insert(
            "gearing_unit".to_string(),
            create_machine("gearing_unit", 1, 10),
        );

        let mut visiting = HashSet::new();
        let result = resolve(
            &recipes,
            &recipes_by_output,
            &machines,
            "amethyst_component",
            1,
            &mut visiting,
        );

        match result {
            ProductionNode::Resolved {
                item_id, inputs, ..
            } => {
                assert_eq!(item_id, "amethyst_component");
                assert_eq!(inputs.len(), 2);

                let item_ids: Vec<String> = inputs
                    .iter()
                    .filter_map(|node| match node {
                        ProductionNode::Resolved { item_id, .. } => Some(item_id.clone()),
                        _ => None,
                    })
                    .collect();

                assert!(item_ids.contains(&"amethyst_fiber".to_string()));
                assert!(item_ids.contains(&"origocrust".to_string()));
            }
            _ => panic!("Expected Resolved node for amethyst_component"),
        }
    }

    #[test]
    fn test_cycle_avoidance() {
        // origocrust can be made from originium_ore or from origocrust_powder (which comes from origocrust)
        let recipe_normal = create_recipe(
            "origocrust",
            "refining_unit",
            vec![("originium_ore", 1)],
            vec![("origocrust", 1)],
        );
        let recipe_powder = create_recipe(
            "origocrust",
            "refining_unit",
            vec![("origocrust_powder", 1)],
            vec![("origocrust", 1)],
        );

        let mut recipes = HashMap::new();
        recipes.insert(
            "origocrust@refining_unit[originium_ore:1]".to_string(),
            recipe_normal,
        );
        recipes.insert(
            "origocrust@refining_unit[origocrust_powder:1]".to_string(),
            recipe_powder,
        );

        let mut recipes_by_output = HashMap::new();
        recipes_by_output.insert(
            "origocrust".to_string(),
            vec![
                "origocrust@refining_unit[originium_ore:1]".to_string(),
                "origocrust@refining_unit[origocrust_powder:1]".to_string(),
            ],
        );

        let mut machines = HashMap::new();
        machines.insert(
            "refining_unit".to_string(),
            create_machine("refining_unit", 1, 5),
        );

        let mut visiting = HashSet::new();
        let result = resolve(
            &recipes,
            &recipes_by_output,
            &machines,
            "origocrust",
            1,
            &mut visiting,
        );

        // Should select the originium_ore recipe to avoid potential cycle
        match result {
            ProductionNode::Resolved {
                item_id,
                machine_id,
                ..
            } => {
                assert_eq!(item_id, "origocrust");
                assert_eq!(machine_id, "refining_unit");
            }
            _ => panic!("Expected Resolved node"),
        }
    }

    #[test]
    fn test_unresolved_when_no_recipe() {
        let recipes = HashMap::new();
        let recipes_by_output = HashMap::new();
        let machines = HashMap::new();

        let mut visiting = HashSet::new();
        let result = resolve(
            &recipes,
            &recipes_by_output,
            &machines,
            "unknown_material",
            10,
            &mut visiting,
        );

        match result {
            ProductionNode::Unresolved { item_id, amount } => {
                assert_eq!(item_id, "unknown_material");
                assert_eq!(amount, 10);
            }
            _ => panic!("Expected Unresolved node"),
        }
    }
}
