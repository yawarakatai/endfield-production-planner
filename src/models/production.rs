use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub enum ProductionNode {
    Resolved {
        item_id: String,
        machine_id: String,
        amount: u32,
        machine_count: u32,
        power_usage: u32,
        load: f64,
        inputs: Vec<ProductionNode>,
    },
    Unresolved {
        item_id: String,
        amount: u32,
    },
    Cycle {
        item_id: String,
        amount: u32,
    },
}

impl ProductionNode {
    pub fn is_source(&self) -> bool {
        match self {
            ProductionNode::Resolved { inputs, .. } => inputs.is_empty(),
            _ => false,
        }
    }

    pub fn total_power(&self) -> u32 {
        match self {
            ProductionNode::Resolved {
                power_usage,
                inputs,
                ..
            } => power_usage + inputs.iter().map(|n| n.total_power()).sum::<u32>(),
            _ => 0,
        }
    }

    pub fn total_source_materials(&self) -> HashMap<String, u32> {
        let mut totals = HashMap::new();
        self.collect_totals(&mut totals);
        totals
    }

    fn collect_totals(&self, totals: &mut HashMap<String, u32>) {
        match self {
            ProductionNode::Resolved {
                item_id,
                amount,
                inputs,
                ..
            } => {
                if self.is_source() {
                    *totals.entry(item_id.clone()).or_insert(0) += amount;
                } else {
                    for child in inputs {
                        child.collect_totals(totals);
                    }
                }
            }
            ProductionNode::Unresolved { item_id, amount } => {
                *totals.entry(item_id.clone()).or_insert(0) += amount;
            }
            ProductionNode::Cycle { item_id, amount } => {
                *totals.entry(item_id.clone()).or_insert(0) += amount;
            }
        }
    }
}
