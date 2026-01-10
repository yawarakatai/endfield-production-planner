use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

// ---------------------------
// Data Structures
// ---------------------------

#[derive(Debug, Deserialize)]
struct RecipeDef {
    id: String,
    by: Vec<String>,
    time: u32,
    #[serde(default)]
    inputs: HashMap<String, u32>,
    #[serde(default)]
    outputs: HashMap<String, u32>,
}

#[derive(Debug, Deserialize)]
struct MachineDef {
    id: String,
    /// Crafting speed percentage (100 = 1.0x, 50 = 0.5x, 200 = 2.0x)
    // speed_percent: u32,
    power_usage: u32,
}

// Wrapper for TOML structure [[recipes]] / [[machines]]
#[derive(Debug, Deserialize)]
struct RecipeConfig {
    recipes: Vec<RecipeDef>,
}

#[derive(Debug, Deserialize)]
struct MachineConfig {
    machines: Vec<MachineDef>,
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
    let recipes: HashMap<String, RecipeDef> = recipe_config
        .recipes
        .into_iter()
        .map(|r| (r.id.clone(), r))
        .collect();

    let machines: HashMap<String, MachineDef> = machine_config
        .machines
        .into_iter()
        .map(|m| (m.id.clone(), m))
        .collect();

    println!(
        "Loaded {} recipes and {} machines.\n",
        recipes.len(),
        machines.len()
    );

    // ---------------------------
    // Example Calculation
    // ---------------------------

    // Case 1: Mining 'Originium' with 'Electric Miner 2'
    let recipe_id = "pure_originium_ore";
    let machine_id = "electric_mining_rig_mk2";

    if let (Some(recipe), Some(machine)) = (recipes.get(recipe_id), machines.get(machine_id)) {
        calculate_production(recipe, machine);
    }

    // Case 2: Assembling 'Iron Parts' with 'Assembler 1' (Slower machine)
    let recipe_id = "origocrust";
    let machine_id = "refining_unit";

    if let (Some(recipe), Some(machine)) = (recipes.get(recipe_id), machines.get(machine_id)) {
        calculate_production(recipe, machine);
    }
}

/// Calculate actual time using integer math
fn calculate_production(recipe: &RecipeDef, machine: &MachineDef) {
    println!("--- Calculation: {} + {} ---", recipe.id, machine.id);
    if recipe.by.contains(&machine.id) {
        println!("Total Time: {:} sec", recipe.time);
        println!("Total Power Usage: {} ", machine.power_usage);
    } else {
        println!("Error: This ")
    }
}
