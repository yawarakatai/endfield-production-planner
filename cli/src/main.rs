use std::{collections::HashSet, fs};

use resource_calculator_core::config::GameData;
use resource_calculator_core::constants::{MACHINE_DEFINITION_PATH, RECIPE_DEFINITION_PATH};
use resource_calculator_core::error::ProductionError;
use resource_calculator_core::output::print_summary;
use resource_calculator_core::planner::plan_production;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recipes = fs::read_to_string(RECIPE_DEFINITION_PATH)?;
    let machines = fs::read_to_string(MACHINE_DEFINITION_PATH)?;

    let data = GameData::new(&recipes, &machines)?;

    println!(
        "Loaded {} recipes and {} machines.\n",
        data.recipes.len(),
        data.machines.len()
    );

    let item_id = "cryston_component";
    let amount = 12; // per minute

    if !data.recipes_by_output.contains_key(item_id) {
        return Err(Box::new(ProductionError::RecipeNotFound(
            item_id.to_string(),
        )));
    }

    let mut visiting = HashSet::new();

    let node = plan_production(
        &data.recipes,
        &data.recipes_by_output,
        &data.machines,
        item_id,
        amount,
        &mut visiting,
    );

    print_summary(&node);

    Ok(())
}
