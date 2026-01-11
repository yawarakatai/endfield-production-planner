use leptos::prelude::*;
use resource_calculator_core::config::GameData;
use resource_calculator_core::i18n::{Locale, Localizer};
use resource_calculator_core::models::ProductionNode;
use resource_calculator_core::planner::plan_production;
use std::collections::{HashMap, HashSet};

fn main() {
    leptos::mount::mount_to_body(|| view! { <App/> })
}

#[component]
fn app() -> impl IntoView {
    // Load static data which is executed once on launch
    let recipes_str = include_str!("../../res/recipes.toml");
    let machines_str = include_str!("../../res/machines.toml");
    let game_data = GameData::new(recipes_str, machines_str).expect("Failed to load data");

    // Load locales
    let en_locale = include_str!("../../res/locales/en.toml");
    let ja_locale = include_str!("../../res/locales/ja.toml");

    let localizers: HashMap<Locale, Localizer> = [
        (Locale::English, Localizer::new(en_locale).expect("Failed to load English locale")),
        (Locale::Japanese, Localizer::new(ja_locale).expect("Failed to load Japanese locale")),
    ]
    .into_iter()
    .collect();

    let mut all_items: Vec<String> = game_data.recipes_by_output.keys().cloned().collect();
    all_items.sort();

    // Define signals
    let (target_amount, set_target_amount) = signal(1); // Default: 1
    let (search_query, set_search_query) = signal(String::new());
    let (selected_item, set_selected_item) = signal(
        all_items
            .first()
            .cloned()
            .unwrap_or_else(|| "originium_ore".to_string()),
    );
    let (current_locale, set_current_locale) = signal(Locale::English);

    // Create a memo for the current localizer
    let current_localizer = Memo::new(move |_| {
        localizers.get(&current_locale.get()).unwrap().clone()
    });

    // Filter item list by a query
    let filtered_items = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            all_items.clone()
        } else {
            all_items
                .iter()
                .filter(|item| item.to_lowercase().contains(&query))
                .cloned()
                .collect()
        }
    };

    // Re-calculate the production plan everytime when the input value change
    let production_plan = Memo::new(move |_| {
        let item_id = selected_item.get();
        let amount = target_amount.get();
        let mut visiting = HashSet::new();

        plan_production(
            &game_data.recipes,
            &game_data.recipes_by_output,
            &game_data.machines,
            &item_id,
            amount, // u32
            &mut visiting,
        )
    });

    //  Construct view
    view! {
        <div class="app-container">

            // Left sidebar
            <div class="sidebar">
                <div class="settings-panel">
                    <h3>{move || current_localizer.get().get_ui("settings")}</h3>

                    // Language selector
                    <div class="form-group">
                        <label class="form-label">"Language"</label>
                        <select
                            class="form-input"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                if let Some(locale) = Locale::from_code(&value) {
                                    set_current_locale.set(locale);
                                }
                            }
                        >
                            <option value="en" selected=move || current_locale.get() == Locale::English>
                                "English"
                            </option>
                            <option value="ja" selected=move || current_locale.get() == Locale::Japanese>
                                "日本語"
                            </option>
                        </select>
                    </div>

                    // Input value
                    <div class="form-group">
                        <label class="form-label">{move || current_localizer.get().get_ui("amount_per_min")}</label>
                        <input
                            type="number"
                            min="1"
                            prop:value=move || target_amount.get()
                            on:input=move |ev| {
                                if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                    set_target_amount.set(val);
                                }
                            }
                            class="form-input"
                        />
                    </div>

                    // Item search
                    <div>
                        <label class="form-label">{move || current_localizer.get().get_ui("search_item")}</label>
                        <input
                            type="text"
                            placeholder=move || current_localizer.get().get_ui("search_placeholder")
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            class="form-input"
                        />
                    </div>
                </div>

                // Item list
                <div class="item-list">
                     <For
                        each=filtered_items
                        key=|item| item.clone()
                        children=move |item| {
                            let item_for_click = item.clone();
                            let item_for_class = item.clone();
                            let item_for_display = item.clone();

                            let on_click = move |_| set_selected_item.set(item_for_click.clone());

                            view! {
                                <div
                                    on:click=on_click
                                    class=move || {
                                        let is_selected = selected_item.get() == item_for_class;
                                        if is_selected {
                                            "item-list-entry selected"
                                        } else {
                                            "item-list-entry"
                                        }
                                    }
                                >
                                    {move || current_localizer.get().get_item(&item_for_display)}
                                </div>
                            }
                        }
                    />
                   </div>
                </div>

            // Main content
            <div class="main-content">
                <h1>{move || current_localizer.get().get_ui("production_plan")}</h1>

                // Total values
                <div class="summary-container">

                    // Raw Materials
                    <div class="summary-card">
                        <h4>{move || current_localizer.get().get_ui("total_raw_materials")}</h4>
                        {move || {
                            let localizer = current_localizer.get();
                            let node = production_plan.get();
                            let mut materials: Vec<_> = node.total_source_materials().into_iter().collect();
                            materials.sort_by(|a, b| a.0.cmp(&b.0));

                            if materials.is_empty() {
                                view! { <div class="empty">{localizer.get_ui("none")}</div> }.into_any()
                            } else {
                                view! {
                                    <ul>
                                        {materials.into_iter().map(|(name, count)| {
                                            let display_name = localizer.get_item(&name);
                                            view! { <li>{display_name} ": " <strong>{count}</strong></li> }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }
                        }}
                    </div>

                    // Machines
                    <div class="summary-card">
                        <h4>{move || current_localizer.get().get_ui("total_machines")}</h4>
                        {move || {
                            let localizer = current_localizer.get();
                            let node = production_plan.get();
                            let mut machines: Vec<_> = node.total_machines().into_iter().collect();
                            machines.sort_by(|a, b| a.0.cmp(&b.0));

                            if machines.is_empty() {
                                view! { <div class="empty">{localizer.get_ui("none")}</div> }.into_any()
                            } else {
                                view! {
                                    <ul>
                                        {machines.into_iter().map(|(name, count)| {
                                            let display_name = localizer.get_machine(&name);
                                            view! { <li>{display_name} ": " <strong>{count}</strong></li> }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }
                        }}
                    </div>

                    // Power
                    <div class="summary-card power">
                        <h4>{move || current_localizer.get().get_ui("total_power")}</h4>
                        <div class="power-value">
                            {move || production_plan.get().total_power()}
                            <span class="power-unit">{move || current_localizer.get().get_ui("power_unit")}</span>
                        </div>
                    </div>
                </div>

                // Tree view
                <div class="target-info">
                    <p>
                        {move || current_localizer.get().get_ui("target")} ": " <strong>{move || current_localizer.get().get_item(&selected_item.get())}</strong>
                        " x" {move || target_amount.get()} {move || current_localizer.get().get_ui("per_min")}
                    </p>
                </div>

                <div class="production-tree">
                    {move || view! { <TreeView node=production_plan.get() localizer=current_localizer.get() /> }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn tree_view(node: ProductionNode, localizer: Localizer) -> impl IntoView {
    match node {
        ProductionNode::Resolved {
            item_id,
            machine_id,
            amount,
            machine_count,
            inputs,
            ..
        } => {
            let item_name = localizer.get_item(&item_id);
            let machine_name = localizer.get_machine(&machine_id);
            let localizer_clone = localizer.clone();

            view! {
                <div class="tree-node">
                    <div class="node-content">
                        <span class="connector">"├"</span>
                        <strong>{item_name}</strong>
                        " x"{amount}
                        <span class="machine-info">
                            " (" {machine_name} " x" {machine_count} ")"
                        </span>
                    </div>
                    {
                        inputs.into_iter().map(move |child| {
                            view! { <TreeView node=child localizer=localizer_clone.clone() /> }
                        }).collect_view()
                    }
                </div>
            }
            .into_any()
        }
        ProductionNode::Unresolved { item_id, amount } => {
            let item_name = localizer.get_item(&item_id);
            let missing_text = localizer.get_ui("missing_recipe");

            view! {
                <div class="tree-node">
                     <div class="node-content missing">
                        <span class="connector">"x"</span>
                        {item_name} " x" {amount} " [" {missing_text} "]"
                    </div>
                </div>
            }
            .into_any()
        }
    }
}
