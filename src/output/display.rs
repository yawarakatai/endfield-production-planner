use crate::models::ProductionNode;

fn print_production_tree(node: &ProductionNode, depth: usize) {
    let indent = "  ".repeat(depth);

    match node {
        ProductionNode::Resolved {
            item_id,
            machine_id,
            amount,
            machine_count,
            inputs,
            ..
        } => {
            if node.is_source() {
                println!(
                    "{}[Source] {} x{} (via: {} x{})",
                    indent, item_id, amount, machine_id, machine_count
                );
            } else {
                println!(
                    "{}[Craft] {} x{} (via: {} x{})",
                    indent, item_id, amount, machine_id, machine_count
                );

                for child in inputs {
                    print_production_tree(child, depth + 1)
                }
            }
        }
        ProductionNode::Unresolved { item_id, .. } => {
            println!("{}[MISSING] No recipe for {}", indent, item_id)
        }
        ProductionNode::Cycle { item_id, .. } => {
            println!("{}[CYCLE!] Loop detected at {}", indent, item_id);
        }
    }
}

pub fn print_summary(node: &ProductionNode) {
    println!("--- Production Line Tree ---");
    print_production_tree(node, 0);

    println!("\nTotal Raw Materials Needed:");
    for (item, count) in node.total_source_materials() {
        println!(" - {}: {}", item, count);
    }

    println!("\nTotal Machines Needed:");
    for (machine, count) in node.total_machines() {
        println!(" - {}: {}", machine, count);
    }

    println!("\nTotal Power Needed: {}", node.total_power());
}
