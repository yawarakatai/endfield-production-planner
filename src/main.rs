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
