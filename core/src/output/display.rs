use crate::models::ProductionNode;

fn print_node_recursive(node: &ProductionNode, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = if is_last { "    " } else { "│   " };

    let node_info = match node {
        ProductionNode::Resolved {
            item_id,
            machine_id,
            amount,
            machine_count,
            ..
        } => {
            format!(
                "{} x{} [{} x{}]",
                item_id, amount, machine_id, machine_count
            )
        }
        ProductionNode::Unresolved { item_id, .. } => {
            format!("{} [MISSING RECIPE]", item_id)
        }
    };

    println!("{}{}{}", prefix, connector, node_info);

    if let ProductionNode::Resolved { inputs, .. } = node {
        let count = inputs.len();
        for (i, child) in inputs.iter().enumerate() {
            let is_last_child = i == count - 1;
            print_node_recursive(child, &format!("{}{}", prefix, child_prefix), is_last_child);
        }
    }
}

pub fn print_summary(node: &ProductionNode) {
    println!("--- Production Line Tree ---");

    match node {
        ProductionNode::Resolved {
            item_id,
            machine_id,
            amount,
            machine_count,
            inputs,
            ..
        } => {
            println!(
                "{} x{} [{} x{}]",
                item_id, amount, machine_id, machine_count
            );

            let count = inputs.len();
            for (i, child) in inputs.iter().enumerate() {
                print_node_recursive(child, "", i == count - 1);
            }
        }
        _ => println!("Invalid root node"),
    }

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
