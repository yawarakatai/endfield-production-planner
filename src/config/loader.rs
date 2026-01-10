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

        let recipes = recipe_config
            .recipes
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect();

        let machines = machine_config
            .machines
            .into_iter()
            .map(|m| (m.id.clone(), m))
            .collect();

        Ok(GameData { recipes, machines })
    }
}
