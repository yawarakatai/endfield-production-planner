use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

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
    /// Crafting speed percentage (100 = 1.0x, 50 = 0.5x, 200 = 2.0x)
    // speed_percent: u32,
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

    calculate_production(&recipes, &machines, "sc_valley_battery", 0);
}

fn calculate_production(
    recipes: &HashMap<String, Recipe>,
    machines: &HashMap<String, Machine>,
    id: &str,
    depth: usize,
) {
    let indent = "  ".repeat(depth);

    if depth > 20 {
        println!("{}Stop recursion by hitting the depth limit", indent);
        return;
    }

    if let Some(recipe) = recipes.get(id) {
        if recipe.inputs.is_empty() {
            for machine in &recipe.by {
                println!("{}Mine {} by {}", indent, recipe.id, machine);
            }
            return;
        }

        for machine in &recipe.by {
            println!("{}Craft {} by {}", indent, recipe.id, machine);
        }

        for (input_item_id, count) in &recipe.inputs {
            println!("{}  - Need {} x{}", indent, input_item_id, count);

            calculate_production(recipes, machines, input_item_id, depth + 1);
        }
    } else {
        println!("{}? No recipe found for '{}'", indent, id);
    }
}
