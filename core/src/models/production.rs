use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
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
}

impl ProductionNode {
    pub fn is_source(&self) -> bool {
        match self {
            ProductionNode::Resolved { inputs, .. } => inputs.is_empty(),
            _ => false,
        }
    }

    pub fn utilization(&self) -> u32 {
        match self {
            ProductionNode::Resolved { load, .. } => {
                (*load * 100.0).round().clamp(0.0, 100.0) as u32
            }
            _ => 0,
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
        self.collect_totals(|node| match node {
            ProductionNode::Resolved {
                item_id, amount, ..
            } => {
                if node.is_source() {
                    Some((item_id.clone(), *amount))
                } else {
                    None
                }
            }
            ProductionNode::Unresolved { item_id, amount } => Some((item_id.clone(), *amount)),
        })
    }

    pub fn total_machines(&self) -> HashMap<String, u32> {
        self.collect_totals(|node| match node {
            ProductionNode::Resolved {
                machine_id,
                machine_count,
                ..
            } if !machine_id.is_empty() => Some((machine_id.clone(), *machine_count)),
            _ => None,
        })
    }

    fn collect_totals<F>(&self, extract: F) -> HashMap<String, u32>
    where
        F: Fn(&ProductionNode) -> Option<(String, u32)> + Copy,
    {
        let mut totals = HashMap::new();
        self.collect_totals_recursive(&mut totals, extract);
        totals
    }

    fn collect_totals_recursive<F>(&self, totals: &mut HashMap<String, u32>, extract: F)
    where
        F: Fn(&ProductionNode) -> Option<(String, u32)> + Copy,
    {
        if let Some((key, value)) = extract(self) {
            *totals.entry(key).or_insert(0) += value;
        }

        if let ProductionNode::Resolved { inputs, .. } = self {
            for child in inputs {
                child.collect_totals_recursive(totals, extract);
            }
        }
    }
}
