use endfield_planner_core::config::GameData;
use endfield_planner_core::i18n::{Locale, Localizer};
use endfield_planner_core::models::ProductionNode;
use endfield_planner_core::planner::plan_production;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::components::tree_view::TreeView;
use crate::utils::localization::get_localized_name;

#[component]
pub fn app() -> impl IntoView {
    // Load static data which is executed once on launch
    let recipes_str = include_str!("../../../res/recipes.toml");
    let machines_str = include_str!("../../../res/machines.toml");
    let game_data = GameData::new(recipes_str, machines_str).expect("Failed to load data");

    // Load locales
    let en_locale = include_str!("../../../res/locales/en.toml");
    let ja_locale = include_str!("../../../res/locales/ja.toml");

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

        let mut items: Vec<String> = if query.is_empty() {
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
        };

        items.sort_by(|a, b| {
            let reading_a = localizer.get_reading(a);
            let reading_b = localizer.get_reading(b);
            reading_a.cmp(&reading_b)
        });

        items
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
                                        machine_ids_store.with_value(|machine_ids| {
                                            get_localized_name(&item_id_for_display, &localizer, machine_ids)
                                        })
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
                        <div class="summary-card-content">
                            {move || {
                                let localizer = current_localizer.get();
                                let node = production_plan.get();
                                let total_power = node.total_power();
                                let total_machines: u32 = node.total_machines().values().sum();
                                let utilization_rate = node.utilization();

                                view! {
                                    <ul>
                                        <li>
                                            <span>{localizer.get_ui("power_usage")}</span>
                                            <strong>{total_power}</strong>
                                        </li>
                                        <li>
                                            <span>{localizer.get_ui("total_machine_count")}</span>
                                            <strong>{total_machines} " " {localizer.get_ui("machine_unit")}</strong>
                                        </li>
                                        <li>
                                            <span>{localizer.get_ui("utilization_rate")}</span>
                                            <strong>{utilization_rate} " " %</strong>
                                        </li>
                                    </ul>
                                }
                            }}
                        </div>
                    </div>
                </div>

                // Tree view
                <div class="production-group">
                    <div class="target-info">
                        <p>
                            {move || current_localizer.get().get_ui("target")} ": " <strong>{move || {
                                let localizer = current_localizer.get();
                                let item_id = selected_item.get();
                                machine_ids_store.with_value(|machine_ids| {
                                    get_localized_name(&item_id, &localizer, machine_ids)
                                })
                            }}</strong>
                            " x" {move || target_amount.get()} {move || current_localizer.get().get_ui("per_min")}
                        </p>
                    </div>

                    <div class="production-tree">
                        {move || {
                            let node = production_plan.get();
                            let localizer = current_localizer.get();
                            match &node {
                                ProductionNode::Resolved { item_id, machine_id, amount, machine_count, inputs, .. } => {
                                    let item_name = machine_ids_store.with_value(|machine_ids| {
                                        get_localized_name(item_id, &localizer, machine_ids)
                                    });
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
                                                    {machine_name} " ×" {*machine_count}
                                                </span>
                                            </div>
                                            {
                                                inputs.clone().into_iter().enumerate().map(move |(i, child)| {
                                                    let is_last = i == child_count - 1;
                                                    view! {
                                                        <TreeView
                                                            node=child
                                                            localizer=localizer.clone()
                                                            machine_ids=machine_ids_store
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
                                    let item_name = machine_ids_store.with_value(|machine_ids| {
                                        get_localized_name(item_id, &localizer, machine_ids)
                                    });
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
        </div>
    }
}
