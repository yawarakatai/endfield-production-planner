//! Production planning module for Endfield Production Planner.

mod calculator;
mod dependency_resolver;
mod recipe_selector;

pub use calculator::ProductionCalculation;

use crate::models::{Machine, ProductionNode, Recipe};
use std::collections::{HashMap, HashSet};

/// Plans the production tree for a target item.
///
/// This is the main entry point for production planning.
/// See `dependency_resolver::resolve` for implementation details.
pub fn plan_production(
    recipes: &HashMap<String, Recipe>,
    recipes_by_output: &HashMap<String, Vec<String>>,
    machines: &HashMap<String, Machine>,
    item_id: &str,
    amount: u32,
    visiting: &mut HashSet<String>,
) -> ProductionNode {
    dependency_resolver::resolve(
        recipes,
        recipes_by_output,
        machines,
        item_id,
        amount,
        visiting,
    )
}
