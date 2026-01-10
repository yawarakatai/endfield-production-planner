mod config;
mod constants;
mod error;
mod models;
mod output;
mod planner;

use std::collections::HashSet;

use config::GameData;
use error::ProductionError;
use output::print_summary;
use planner::plan_production;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = GameData::load()?;

    println!(
        "Loaded {} recipes and {} machines.\n",
        data.recipes.len(),
        data.machines.len()
    );

    let item_name = "sc_valley_battery";
    let production_goal = 12;

    if !data.recipes.contains_key(item_name) {
        return Err(Box::new(ProductionError::RecipeNotFound(
            item_name.to_string(),
        )));
    }

    let mut visiting = HashSet::new();

    let node = plan_production(
        &data.recipes,
        &data.machines,
        item_name,
        production_goal,
        &mut visiting,
    );

    print_summary(&node);

    Ok(())
}
