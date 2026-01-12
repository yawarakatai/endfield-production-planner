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
        .unwrap_or_else(|| "manual".to_string());

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
    }
}
