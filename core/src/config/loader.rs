use crate::error::ProductionError;
use crate::models::{Machine, Recipe};
use serde::Deserialize;
use std::collections::HashMap;

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
    pub fn new(recipes_content: &str, machines_content: &str) -> Result<Self, ProductionError> {
        let recipe_config: RecipeConfig = toml::from_str(&recipes_content)
            .map_err(|e| ProductionError::ParseError(format!("recipes.toml: {}", e)))?;
        let machine_config: MachineConfig = toml::from_str(&machines_content)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_toml() {
        let recipes_toml = r#"
[[recipes]]
id = "origocrust"
by = "refining_unit"
time = 2
out = 1
[recipes.inputs]
originium_ore = 1
"#;

        let machines_toml = r#"
[[machines]]
id = "refining_unit"
tier = 1
power = 5
"#;

        let result = GameData::new(recipes_toml, machines_toml);
        assert!(result.is_ok());

        let data = result.unwrap();
        assert_eq!(data.recipes.len(), 1);
        assert_eq!(data.machines.len(), 1);
    }

    #[test]
    fn test_parse_invalid_toml() {
        let invalid_recipes_toml = r#"
[[recipes
id = "origocrust"
this is not valid toml
"#;

        let machines_toml = r#"
[[machines]]
id = "refining_unit"
tier = 1
power = 5
"#;

        let result = GameData::new(invalid_recipes_toml, machines_toml);
        assert!(result.is_err());

        match result {
            Err(ProductionError::ParseError(msg)) => {
                assert!(msg.contains("recipes.toml"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_recipes_by_output_grouping() {
        let recipes_toml = r#"
[[recipes]]
id = "originium_ore"
by = "portable_originium_rig"
time = 2
out = 1
is_source = true

[[recipes]]
id = "originium_ore"
by = "electric_mining_rig"
time = 2
out = 1
is_source = true

[[recipes]]
id = "origocrust"
by = "refining_unit"
time = 2
out = 1
[recipes.inputs]
originium_ore = 1
"#;

        let machines_toml = r#"
[[machines]]
id = "portable_originium_rig"
tier = 1
power = 0

[[machines]]
id = "electric_mining_rig"
tier = 2
power = 5

[[machines]]
id = "refining_unit"
tier = 1
power = 5
"#;

        let result = GameData::new(recipes_toml, machines_toml);
        assert!(result.is_ok());

        let data = result.unwrap();

        // Both originium_ore recipes should be grouped under "originium_ore"
        let ore_recipes = data.recipes_by_output.get("originium_ore");
        assert!(ore_recipes.is_some());
        assert_eq!(ore_recipes.unwrap().len(), 2);

        // origocrust should have only one recipe
        let crust_recipes = data.recipes_by_output.get("origocrust");
        assert!(crust_recipes.is_some());
        assert_eq!(crust_recipes.unwrap().len(), 1);

        // Total recipes should be 3
        assert_eq!(data.recipes.len(), 3);
    }
}
