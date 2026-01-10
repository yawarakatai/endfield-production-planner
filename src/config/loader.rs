use crate::constants::{MACHINE_DEFINITION_PATH, RECIPE_DEFINITION_PATH};
use crate::error::ProductionError;
use crate::models::{Machine, Recipe};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
struct RecipeConfig {
    recipes: Vec<Recipe>,
}

#[derive(Debug, Deserialize)]
struct MachineConfig {
    machines: Vec<Machine>,
}

pub struct GameData {
    pub recipes: HashMap<String, Recipe>,
    pub recipes_by_output: HashMap<String, Vec<String>>,
    pub machines: HashMap<String, Machine>,
}

impl GameData {
    pub fn load() -> Result<Self, ProductionError> {
        let recipes_str = fs::read_to_string(RECIPE_DEFINITION_PATH)
            .map_err(|_| ProductionError::FileNotFound(RECIPE_DEFINITION_PATH.to_string()))?;
        let machines_str = fs::read_to_string(MACHINE_DEFINITION_PATH)
            .map_err(|_| ProductionError::FileNotFound(MACHINE_DEFINITION_PATH.to_string()))?;

        let recipe_config: RecipeConfig = toml::from_str(&recipes_str)
            .map_err(|e| ProductionError::ParseError(format!("recipes.toml: {}", e)))?;
        let machine_config: MachineConfig = toml::from_str(&machines_str)
            .map_err(|e| ProductionError::ParseError(format!("machines.toml: {}", e)))?;

        let mut recipes = HashMap::new();
        let mut recipes_by_output: HashMap<String, Vec<String>> = HashMap::new();

        for mut r in recipe_config.recipes {
            r.normalize();

            let unique_id = r.compute_unique_id();
            let output_item = r.id.clone();

            recipes_by_output
                .entry(output_item)
                .or_default()
                .push(unique_id.clone());

            recipes.insert(unique_id, r);
        }

        let machines = machine_config
            .machines
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        Ok(GameData {
            recipes,
            recipes_by_output,
            machines,
        })
    }
}
