//! Production calculation utilities.

use crate::constants::PRODUCTION_TIME_WINDOW;
use crate::models::{Machine, Recipe};

/// Result of production calculations for a single recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionCalculation {
    /// Number of crafting operations needed per time window.
    pub required_crafts: f64,
    /// Number of machines needed (rounded up).
    pub machine_count: u32,
    /// Machine utilization ratio (0.0 to 1.0).
    pub load: f64,
    /// Total power consumption for all machines.
    pub power_usage: u32,
}

/// Calculates production requirements for a recipe.
///
/// # Arguments
/// * `recipe` - The recipe to calculate for
/// * `machine` - The machine used (None for manual crafting)
/// * `target_amount` - Desired output per time window
/// * `item_id` - The target item ID to look up output count
pub fn calculate(
    recipe: &Recipe,
    machine: Option<&Machine>,
    target_amount: u32,
    item_id: &str,
) -> ProductionCalculation {
    let power = machine.map(|m| m.power).unwrap_or(0);
    let output_per_craft = *recipe.outputs.get(item_id).unwrap_or(&1) as f64;
    let recipe_time = recipe.time as f64;

    let required_crafts = target_amount as f64 / output_per_craft;
    let required_machines = recipe_time * required_crafts / PRODUCTION_TIME_WINDOW;
    let machine_count = required_machines.ceil() as u32;

    let load = if machine_count > 0 {
        required_machines / machine_count as f64
    } else {
        1.0
    };

    let power_usage = (power as u64 * machine_count as u64).min(u32::MAX as u64) as u32;

    ProductionCalculation {
        required_crafts,
        machine_count,
        load,
        power_usage,
    }
}
