use leptos::prelude::*;
use resource_calculator_core::i18n::Localizer;
use resource_calculator_core::models::ProductionNode;
use std::collections::HashSet;

use crate::utils::localization::get_localized_name;

#[component]
pub fn tree_view(
    node: ProductionNode,
    localizer: Localizer,
    machine_ids: StoredValue<HashSet<String>>,
    #[prop(default = true)] is_last: bool,
    #[prop(default = vec![])] prefix: Vec<bool>,
) -> impl IntoView {
    match node {
        ProductionNode::Resolved {
            item_id,
            machine_id,
            amount,
            machine_count,
            inputs,
            ..
        } => {
            let item_name = machine_ids.with_value(|ids| {
                get_localized_name(&item_id, &localizer, ids)
            });
            let machine_name = localizer.get_machine(&machine_id);
            let localizer_clone = localizer.clone();
            let child_count = inputs.len();

            // Build the prefix string for display
            let prefix_str: String = prefix
                .iter()
                .map(|&has_line| if has_line { "│   " } else { "    " })
                .collect();

            let connector = if is_last { "└── " } else { "├── " };

            // Build new prefix for children
            let mut child_prefix = prefix.clone();
            child_prefix.push(!is_last);

            view! {
                <div class="tree-line">
                    <span class="tree-prefix">{prefix_str}</span>
                    <span class="tree-connector">{connector}</span>
                    <span class="tree-item">
                        <strong>{item_name}</strong>
                        " ×"{amount}
                    </span>
                    <span class="tree-machine">
                        "[" {machine_name} " ×" {machine_count} "]"
                    </span>
                </div>
                {
                    inputs.into_iter().enumerate().map(move |(i, child)| {
                        let is_last_child = i == child_count - 1;
                        let child_prefix_clone = child_prefix.clone();
                        view! {
                            <TreeView
                                node=child
                                localizer=localizer_clone.clone()
                                machine_ids=machine_ids
                                is_last=is_last_child
                                prefix=child_prefix_clone
                            />
                        }
                    }).collect_view()
                }
            }
            .into_any()
        }
        ProductionNode::Unresolved { item_id, amount } => {
            let item_name = machine_ids.with_value(|ids| {
                get_localized_name(&item_id, &localizer, ids)
            });
            let missing_text = localizer.get_ui("missing_recipe");

            let prefix_str: String = prefix
                .iter()
                .map(|&has_line| if has_line { "│   " } else { "    " })
                .collect();

            let connector = if is_last { "└── " } else { "├── " };

            view! {
                <div class="tree-line tree-missing">
                    <span class="tree-prefix">{prefix_str}</span>
                    <span class="tree-connector">{connector}</span>
                    <span class="tree-item">
                        <strong>{item_name}</strong>
                        " ×" {amount}
                    </span>
                    <span class="tree-machine missing">
                        "[" {missing_text} "]"
                    </span>
                </div>
            }
            .into_any()
        }
    }
}
