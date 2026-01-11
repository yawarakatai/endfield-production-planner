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
        (
            Locale::English,
            Localizer::new(en_locale).expect("Failed to load English locale"),
        ),
        (
            Locale::Japanese,
            Localizer::new(ja_locale).expect("Failed to load Japanese locale"),
        ),
    ]
    .into_iter()
    .collect();

    let mut all_items: Vec<String> = game_data.recipes_by_output.keys().cloned().collect();
    all_items.sort();

    let machine_ids: HashSet<String> = game_data.machines.keys().cloned().collect();
    let machine_ids_store = StoredValue::new(machine_ids);

    // Deternime user's language setting to decide initial locale
    let initial_locale = {
        if let Some(window) = web_sys::window() {
            let navigator = window.navigator();

            if let Some(lang) = navigator.language() {
                if lang.starts_with("ja") {
                    Locale::Japanese
                } else {
                    Locale::English
                }
            } else {
                Locale::English
            }
        } else {
            Locale::English
        }
    };

    // Define signals
    let (target_amount, set_target_amount) = signal(1); // Default: 1
    let (search_query, set_search_query) = signal(String::new());

    let (selected_item, set_selected_item) = signal(
        all_items
            .first()
            .cloned()
            .unwrap_or_else(|| "originium_ore".to_string()),
    );
    let (current_locale, set_current_locale) = signal(initial_locale);

    // Create a memo for the current localizer
    let current_localizer =
        Memo::new(move |_| localizers.get(&current_locale.get()).unwrap().clone());

    // Filter item list by a query (search both ID and localized name)
    let filtered_items = move || {
        let query = search_query.get().to_lowercase();
        let localizer = current_localizer.get();

        if query.is_empty() {
            all_items.clone()
        } else {
            all_items
                .iter()
                .filter(|item| {
                    // Search by item ID
                    let id_match = item.to_lowercase().contains(&query);
                    // Search by localized name
                    let localized_name = localizer.get_item(item).to_lowercase();
                    let name_match = localized_name.contains(&query);

                    id_match || name_match
                })
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
        <header class="app-header">
            <div class="app-logo">"ENDFIELD PRODUCTION PLANNER"</div>
        </header>

        <div class="app-container">

            // Left sidebar
            <div class="sidebar">
                <div class="settings-panel">
                    <h3>{move || current_localizer.get().get_ui("settings")}</h3>

                    // Language selector
                    <div class="form-group">
                        <label class="form-label">{move || current_localizer.get().get_ui("language")}</label>
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
                            let item_id_for_display = item.clone();

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
                                    {move || {
                                        let localizer = current_localizer.get();
                                        let is_machine = machine_ids_store.with_value(|ids| ids.contains(&item_id_for_display));

                                        if is_machine {
                                            localizer.get_machine(&item_id_for_display)
                                        } else {
                                            localizer.get_item(&item_id_for_display)
                                        }
                                    }}
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
                        <div class="summary-card-content">
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
                    </div>

                    // Machines
                    <div class="summary-card">
                        <h4>{move || current_localizer.get().get_ui("total_machines")}</h4>
                        <div class="summary-card-content">
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
                    {move || {
                        let node = production_plan.get();
                        let localizer = current_localizer.get();
                        match &node {
                            ProductionNode::Resolved { item_id, machine_id, amount, machine_count, inputs, .. } => {
                                let item_name = localizer.get_item(item_id);
                                let machine_name = localizer.get_machine(machine_id);
                                let child_count = inputs.len();
                                view! {
                                    <div class="tree-root">
                                        <div class="tree-line tree-root-line">
                                            <span class="tree-item">
                                                <strong>{item_name}</strong>
                                                " ×"{*amount}
                                            </span>
                                            <span class="tree-machine">
                                                "[" {machine_name} " ×" {*machine_count} "]"
                                            </span>
                                        </div>
                                        {
                                            inputs.clone().into_iter().enumerate().map(move |(i, child)| {
                                                let is_last = i == child_count - 1;
                                                view! {
                                                    <TreeView
                                                        node=child
                                                        localizer=localizer.clone()
                                                        is_last=is_last
                                                        prefix=vec![]
                                                    />
                                                }
                                            }).collect_view()
                                        }
                                    </div>
                                }.into_any()
                            }
                            ProductionNode::Unresolved { item_id, amount } => {
                                let item_name = localizer.get_item(item_id);
                                view! {
                                    <div class="tree-line tree-missing">
                                        <span class="tree-item">{item_name} " ×" {*amount}</span>
                                        <span class="tree-machine missing">"[" {localizer.get_ui("missing_recipe")} "]"</span>
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn tree_view(
    node: ProductionNode,
    localizer: Localizer,
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
            let item_name = localizer.get_item(&item_id);
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
            let item_name = localizer.get_item(&item_id);
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
