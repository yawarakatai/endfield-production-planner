use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

const PRODUCTION_TIME_WINDOW: f64 = 60.0;

// ---------------------------
// Data Structures
// ---------------------------

#[derive(Debug, Deserialize)]
struct Recipe {
    id: String,
    by: Vec<String>,
    time: u32,
    #[serde(default)]
    inputs: HashMap<String, u32>,
    #[serde(default)]
    outputs: HashMap<String, u32>,
}

#[derive(Debug, Deserialize)]
struct Machine {
    id: String,
    tier: u32,
    power: u32,
}

// Wrapper for TOML structure [[recipes]] / [[machines]]
#[derive(Debug, Deserialize)]
struct RecipeConfig {
    recipes: Vec<Recipe>,
}

#[derive(Debug, Deserialize)]
struct MachineConfig {
    machines: Vec<Machine>,
}

#[derive(Debug, Clone, Serialize)]
enum ProductionNode {
    Resolved {
        recipe_id: String,
        machine_id: String,
        amount: u32,
        machine_count: u32,
        power_usage: u32,
        load: f64,
        inputs: Vec<ProductionNode>,
    },
    Unresolved {
        item_id: String,
        amount: u32,
    },
}

impl ProductionNode {
    fn is_source(&self) -> bool {
        match self {
            ProductionNode::Resolved { inputs, .. } => inputs.is_empty(),
            _ => false,
        }
    }

    fn total_power(&self) -> u32 {
        match self {
            ProductionNode::Resolved {
                power_usage,
                inputs,
                ..
            } => power_usage + inputs.iter().map(|n| n.total_power()).sum::<u32>(),
            ProductionNode::Unresolved { .. } => 0,
        }
    }

    fn total_source_materials(&self) -> HashMap<String, u32> {
        let mut totals = HashMap::new();
        self.collect_totals(&mut totals);
        totals
    }

    fn collect_totals(&self, totals: &mut HashMap<String, u32>) {
        match self {
            ProductionNode::Resolved {
                recipe_id,
                amount,
                inputs,
                ..
            } => {
                if self.is_source() {
                    *totals.entry(recipe_id.clone()).or_insert(0) += amount;
                } else {
                    for child in inputs {
                        child.collect_totals(totals);
                    }
                }
            }
            ProductionNode::Unresolved { item_id, amount } => {
                *totals.entry(item_id.clone()).or_insert(0) += amount;
            }
        }
    }
}

// ---------------------------
// Logic
// ---------------------------

fn main() {
    // 1. Load Data
    let recipes_str = fs::read_to_string("res/recipes.toml").expect("Failed to read recipes");
    let machines_str = fs::read_to_string("res/machines.toml").expect("Failed to read machines");

    let recipe_config: RecipeConfig = toml::from_str(&recipes_str).expect("Recipe Parse Error");
    let machine_config: MachineConfig = toml::from_str(&machines_str).expect("Machine Parse Error");

    // Convert to HashMaps for easy lookup (ID -> Data)
    let recipes: HashMap<String, Recipe> = recipe_config
        .recipes
        .into_iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    let machines: HashMap<String, Machine> = machine_config
        .machines
        .into_iter()
        .map(|m| (m.id.clone(), m))
        .collect();

    println!(
        "Loaded {} recipes and {} machines.\n",
        recipes.len(),
        machines.len()
    );

    let item_name = "sc_valley_battery";
    let production_goal = 12;
    let node = plan_production(&recipes, &machines, item_name, production_goal);

    println!("--- Production Line Tree ---");
    print_production_tree(&node, 0);
    println!("Total Raw Materials Needed:");
    for (item, count) in node.total_source_materials() {
        println!(" - {}: {}", item, count);
    }
    println!("Total Power Usage: {}", node.total_power());
}

fn plan_production(
    recipes: &HashMap<String, Recipe>,
    machines: &HashMap<String, Machine>,
    recipe_id: &str,
    amount: u32,
) -> ProductionNode {
    if let Some(recipe) = recipes.get(recipe_id) {
        let selected_machine = recipe
            .by
            .iter()
            .filter_map(|id| machines.get(id))
            .max_by_key(|m| m.tier);

        let (machine_id, power) = match selected_machine {
            Some(m) => (m.id.clone(), m.power),
            None => ("manual".to_string(), 0),
        };

        let amount_f64 = amount as f64;
        let output_per_craft = *recipe.outputs.get(recipe_id).unwrap_or(&1) as f64;
        let craft_time = recipe.time as f64;

        let required_crafts = amount_f64 / output_per_craft;
        let required_machines = craft_time * required_crafts / PRODUCTION_TIME_WINDOW;
        let machine_count = required_machines.ceil() as u32;

        let load = if machine_count > 0 {
            required_machines / machine_count as f64
        } else {
            0.0
        };

        let power_usage = power.checked_mul(machine_count).unwrap_or(0);

        let children: Vec<ProductionNode> = recipe
            .inputs
            .iter()
            .map(|(input_id, input_count)| {
                let sub_amount = *input_count as f64 * required_crafts;
                plan_production(recipes, machines, input_id, sub_amount.ceil() as u32)
            })
            .collect();

        return ProductionNode::Resolved {
            recipe_id: recipe_id.to_string(),
            machine_id: machine_id.to_string(),
            amount,
            machine_count,
            power_usage,
            load,
            inputs: children,
        };
    }

    ProductionNode::Unresolved {
        item_id: recipe_id.to_string(),
        amount,
    }
}

fn print_production_tree(node: &ProductionNode, depth: usize) {
    let indent = "  ".repeat(depth);

    match node {
        ProductionNode::Resolved {
            recipe_id,
            machine_id,
            amount,
            machine_count,
            inputs,
            ..
        } => {
            if node.is_source() {
                println!(
                    "{}[Source] {} x{} (via: {} x{})",
                    indent, recipe_id, amount, machine_id, machine_count
                );
            } else {
                println!(
                    "{}[Craft] {} x{} (via: {} x{})",
                    indent, recipe_id, amount, machine_id, machine_count
                );

                for child in inputs {
                    print_production_tree(child, depth + 1)
                }
            }
        }
        ProductionNode::Unresolved { item_id, .. } => {
            println!("{}[MISSING] No recipe for {}", indent, item_id)
        }
    }
}
