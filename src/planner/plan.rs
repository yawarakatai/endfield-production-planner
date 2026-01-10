use crate::constants::PRODUCTION_TIME_WINDOW;
use crate::models::{Machine, ProductionNode, Recipe};
use std::collections::{HashMap, HashSet};

pub fn plan_production(
    recipes: &HashMap<String, Recipe>,
    recipes_by_outpus: &HashMap<String, Vec<String>>,
    machines: &HashMap<String, Machine>,
    item_id: &str,
    amount: u32,
    visiting: &mut HashSet<String>,
) -> ProductionNode {
    // Detect circular reference
    if visiting.contains(item_id) {
        return ProductionNode::Resolved {
            item_id: item_id.to_string(),
            machine_id: "Loop Source".to_string(),
            amount,
            machine_count: 0,
            power_usage: 0,
            load: 0.0,
            inputs: Vec::new(),
        };
    }

    // Start recording of visit
    visiting.insert(item_id.to_string());

    let selected_recipe = recipes_by_outpus.get(item_id).and_then(|candidates| {
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

                // Priority:
                // First: higher tier
                // Second: lower power
                // Third: sort by id
                tier_a
                    .cmp(&tier_b)
                    .then_with(|| power_b.cmp(&power_a))
                    .then_with(|| recipe_a.id.cmp(&recipe_b.id))
            })
    });

    let result = if let Some(recipe) = selected_recipe {
        let (machine_id, power) = match machines.get(&recipe.by) {
            Some(m) => (m.id.clone(), m.power),
            None => ("manual".to_string(), 0),
        };

        let amount_f64 = amount as f64;
        let output_per_craft = *recipe.outputs.get(item_id).unwrap_or(&1) as f64;
        let recipe_time = recipe.time as f64;

        let required_crafts = amount_f64 / output_per_craft;
        let required_machines = recipe_time * required_crafts / PRODUCTION_TIME_WINDOW;
        let machine_count = required_machines.ceil() as u32;
        let load = if machine_count > 0 {
            required_machines / machine_count as f64
        } else {
            0.0
        };

        let children: Vec<ProductionNode> = recipe
            .inputs
            .iter()
            .map(|(input_id, input_count)| {
                let sub_amount = (*input_count as f64 * required_crafts).ceil() as u32;
                plan_production(
                    recipes,
                    recipes_by_outpus,
                    machines,
                    input_id,
                    sub_amount,
                    visiting,
                )
            })
            .collect();

        ProductionNode::Resolved {
            item_id: item_id.to_string(),
            machine_id,
            amount,
            machine_count,
            load,
            power_usage: (power as u64 * machine_count as u64).min(u32::MAX as u64) as u32,
            inputs: children,
        }
    } else {
        ProductionNode::Unresolved {
            item_id: item_id.to_string(),
            amount,
        }
    };

    // Backtrack
    visiting.remove(item_id);

    result
}
