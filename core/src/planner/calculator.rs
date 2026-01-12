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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_recipe(id: &str, by: &str, time: u32, outputs: Vec<(&str, u32)>) -> Recipe {
        Recipe::new_for_test(
            id.to_string(),
            by.to_string(),
            time,
            HashMap::new(),
            outputs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
            false,
        )
    }

    fn create_machine(id: &str, tier: u32, power: u32) -> Machine {
        Machine {
            id: id.to_string(),
            tier,
            power,
        }
    }

    #[test]
    fn test_machine_count_rounds_up() {
        // origocrust: time=2, out=1
        let recipe = create_recipe("origocrust", "refining_unit", 2, vec![("origocrust", 1)]);
        let machine = create_machine("refining_unit", 1, 5);

        // Required machines = (2 * 31) / 60 = 1.033..., should round up to 2
        let calc = calculate(&recipe, Some(&machine), 31, "origocrust");

        assert_eq!(calc.machine_count, 2);
    }

    #[test]
    fn test_load_calculation() {
        // amethyst_fiber: time=2, out=1
        let recipe = create_recipe("amethyst_fiber", "refining_unit", 2, vec![("amethyst_fiber", 1)]);
        let machine = create_machine("refining_unit", 1, 5);

        // Required machines = (2 * 25) / 60 = 0.8333...
        // Machine count = 1 (rounded up)
        // Load = 0.8333... / 1 = 0.8333...
        let calc = calculate(&recipe, Some(&machine), 25, "amethyst_fiber");

        assert_eq!(calc.machine_count, 1);
        assert!((calc.load - 0.8333333).abs() < 0.0001);
    }

    #[test]
    fn test_power_usage() {
        // ferrium: time=2, out=1, uses grinding_unit with power=20
        let recipe = create_recipe("ferrium", "refining_unit", 2, vec![("ferrium", 1)]);
        let machine = create_machine("refining_unit", 1, 5);

        // Required machines = (2 * 90) / 60 = 3
        // Machine count = 3, power = 5
        // Power usage = 3 * 5 = 15
        let calc = calculate(&recipe, Some(&machine), 90, "ferrium");

        assert_eq!(calc.machine_count, 3);
        assert_eq!(calc.power_usage, 15);
    }

    #[test]
    fn test_required_crafts_with_multiple_output() {
        // carbon from jincao: time=2, out=2
        let recipe = create_recipe("carbon", "refining_unit", 2, vec![("carbon", 2)]);
        let machine = create_machine("refining_unit", 1, 5);

        // Required crafts = 10 / 2 = 5.0
        let calc = calculate(&recipe, Some(&machine), 10, "carbon");

        assert_eq!(calc.required_crafts, 5.0);
    }

    #[test]
    fn test_zero_time_recipe() {
        // Machine construction recipes have time=0
        let recipe = create_recipe("refining_unit", "hand", 0, vec![("refining_unit", 1)]);
        let machine = create_machine("hand", 0, 0);

        // Required machines = (0 * 10) / 60 = 0
        // Machine count = 0 (rounded up from 0)
        // Load should be 1.0 when machine_count is 0
        let calc = calculate(&recipe, Some(&machine), 10, "refining_unit");

        assert_eq!(calc.machine_count, 0);
        assert_eq!(calc.load, 1.0);
        assert_eq!(calc.power_usage, 0);
    }
}
