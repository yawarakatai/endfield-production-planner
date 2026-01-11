use leptos::prelude::*;
use resource_calculator_core::config::GameData;
use resource_calculator_core::models::ProductionNode;
use resource_calculator_core::planner::plan_production;
use std::collections::HashSet;

fn main() {
    leptos::mount::mount_to_body(|| view! { <App/> })
}

#[component]
fn App() -> impl IntoView {
    // 1. 静的データのロード (アプリ起動時に1回だけ実行)
    // ---------------------------------------------------
    let recipes_str = include_str!("../../res/recipes.toml");
    let machines_str = include_str!("../../res/machines.toml");
    // データの所有権をMoveできるようにArc等で包むか、ここでは単純にクローンして使う形にします
    let game_data = GameData::new(recipes_str, machines_str).expect("Failed to load data");

    // 全アイテムのリストを作成してソートしておく
    let mut all_items: Vec<String> = game_data.recipes_by_output.keys().cloned().collect();
    all_items.sort();

    // 2. シグナル（状態）の定義
    // ---------------------------------------------------
    // (値の読み取りアクセサ, 値の書き込みアクセサ) = signal(初期値);
    let (target_amount, set_target_amount) = signal(1); // デフォルト数量: 1
    let (search_query, set_search_query) = signal(String::new()); // 検索ボックスの中身
    let (selected_item, set_selected_item) = signal(
        all_items
            .first()
            .cloned()
            .unwrap_or_else(|| "originium_ore".to_string()),
    ); // 現在選択されているアイテム

    // 3. 派生シグナル（自動計算される値）
    // ---------------------------------------------------

    // A. 検索クエリに基づいてアイテムリストをフィルタリングする
    // `move ||` は、このクロージャ内で外部の変数(all_itemsなど)を使うための記述です
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

    // B. 入力値が変わるたびに生産計画を再計算する
    // Memoを使うと、依存するシグナルが変わった時だけ再実行されます
    let production_plan = Memo::new(move |_| {
        let item_id = selected_item.get();
        let amount = target_amount.get();
        let mut visiting = HashSet::new();

        // ここでCoreの計算ロジックを呼び出す
        plan_production(
            &game_data.recipes,
            &game_data.recipes_by_output,
            &game_data.machines,
            &item_id,
            amount, // u32
            &mut visiting,
        )
    });

    // 4. ビュー（UI）の構築
    // ---------------------------------------------------
    view! {
        <div style="font-family: sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; display: flex; gap: 20px;">

            // --- 左サイドバー: 設定エリア ---
            <div style="width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 15px;">
                <div style="background: #f5f5f5; padding: 15px; border-radius: 8px;">
                    <h3>"Settings"</h3>

                    // 数量入力
                    <div style="margin-bottom: 10px;">
                        <label style="display:block; font-size: 0.9em; margin-bottom: 4px;">"Amount (/min)"</label>
                        <input
                            type="number"
                            min="1"
                            // 値をバインド
                            prop:value=move || target_amount.get()
                            // 入力イベントでシグナルを更新
                            on:input=move |ev| {
                                // event_target_valueで入力文字を取得し、数値にパース
                                if let Ok(val) = event_target_value(&ev).parse::<u32>() {
                                    set_target_amount.set(val);
                                }
                            }
                            style="width: 100%; padding: 5px;"
                        />
                    </div>

                    // アイテム検索
                    <div>
                        <label style="display:block; font-size: 0.9em; margin-bottom: 4px;">"Search Item"</label>
                        <input
                            type="text"
                            placeholder="Type to filter..."
                            prop:value=move || search_query.get()
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                            style="width: 100%; padding: 5px;"
                        />
                    </div>
                </div>

                // アイテムリスト（検索結果）
                <div style="flex-grow: 1; border: 1px solid #ddd; border-radius: 8px; overflow-y: auto; max-height: 70vh;">
                     <For
                        each=filtered_items
                        key=|item| item.clone()
                        children=move |item| {
                            // itemの所有権を複製（クロージャ内で使うため）
                            let item_for_click = item.clone();
                            let item_for_style = item.clone();

                            let on_click = move |_| set_selected_item.set(item_for_click.clone());

                            view! {
                                <div
                                    on:click=on_click
                                    // 【修正後】 style全体を move || で囲み、選択状態が変わるたびに文字列を作り直す
                                    style=move || {
                                        let is_selected = selected_item.get() == item_for_style;
                                        let bg_color = if is_selected { "#e3f2fd" } else { "white" };
                                        let cursor = if is_selected { "default" } else { "pointer" };
                                        format!("padding: 8px 12px; border-bottom: 1px solid #eee; background: {}; cursor: {};", bg_color, cursor)
                                    }
                                >
                                    {item}
                                </div>
                            }
                        }
                    />
                   </div>
                </div>

            // --- メインエリア: ツリー表示 ---
            <div style="flex-grow: 1;">
                <h1 style="margin: 0;">"Production Plan"</h1>

                // --- 追加: 集計情報の表示エリア ---
                <div style="display: flex; flex-wrap: wrap; gap: 20px; margin-bottom: 30px;">

                    // 1. 原材料 (Raw Materials)
                    <div style="flex: 1; min-width: 200px; background: #fafafa; padding: 15px; border-radius: 8px; border: 1px solid #eee;">
                        <h4 style="margin-top: 0; border-bottom: 2px solid #ddd; padding-bottom: 5px;">"Total Raw Materials"</h4>
                        {move || {
                            // 計算結果を取得
                            let node = production_plan.get();
                            let mut materials: Vec<_> = node.total_source_materials().into_iter().collect();
                            materials.sort_by(|a, b| a.0.cmp(&b.0)); // 名前順にソート

                            if materials.is_empty() {
                                view! { <div style="color: #999;">"None"</div> }.into_any()
                            } else {
                                view! {
                                    <ul style="padding-left: 20px; margin: 0;">
                                        {materials.into_iter().map(|(name, count)| {
                                            view! { <li>{name} ": " <strong>{count}</strong></li> }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }
                        }}
                    </div>

                    // 2. 機械 (Machines)
                    <div style="flex: 1; min-width: 200px; background: #fafafa; padding: 15px; border-radius: 8px; border: 1px solid #eee;">
                        <h4 style="margin-top: 0; border-bottom: 2px solid #ddd; padding-bottom: 5px;">"Total Machines"</h4>
                        {move || {
                            let node = production_plan.get();
                            let mut machines: Vec<_> = node.total_machines().into_iter().collect();
                            machines.sort_by(|a, b| a.0.cmp(&b.0));

                            if machines.is_empty() {
                                view! { <div style="color: #999;">"None"</div> }.into_any()
                            } else {
                                view! {
                                    <ul style="padding-left: 20px; margin: 0;">
                                        {machines.into_iter().map(|(name, count)| {
                                            view! { <li>{name} ": " <strong>{count}</strong></li> }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }
                        }}
                    </div>

                    // 3. 電力 (Power)
                    <div style="flex: 1; min-width: 200px; background: #fffbe6; padding: 15px; border-radius: 8px; border: 1px solid #faad14;">
                        <h4 style="margin-top: 0; border-bottom: 2px solid #faad14; padding-bottom: 5px; color: #d48806;">"Total Power"</h4>
                        <div style="font-size: 2em; font-weight: bold; color: #d48806; text-align: center; margin-top: 10px;">
                            {move || production_plan.get().total_power()}
                            <span style="font-size: 0.5em; margin-left: 5px;">"Units"</span>
                        </div>
                    </div>
                </div>

                <div style="margin-bottom: 20px;">
                    <p style="color: #666;">
                        "Target: " <strong>{move || selected_item.get()}</strong>
                        " x" {move || target_amount.get()} "/min"
                    </p>
                </div>

                <div class="production-tree">
                    {move || view! { <TreeView node=production_plan.get() /> }}
                </div>
            </div>
        </div>
    }
}

// TreeViewコンポーネントは前回のまま（変更なし）
#[component]
fn TreeView(node: ProductionNode) -> impl IntoView {
    match node {
        ProductionNode::Resolved {
            item_id,
            machine_id,
            amount,
            machine_count,
            inputs,
            ..
        } => view! {
            <div class="tree-node">
                <div class="node-content">
                    <span class="connector">"├"</span>
                    <strong>{item_id}</strong>
                    " x"{amount}
                    <span style="color: #666; font-size: 0.9em; margin-left: 10px;">
                        " (" {machine_id} " x" {machine_count} ")"
                    </span>
                </div>
                {
                    inputs.into_iter().map(|child| {
                        view! { <TreeView node=child /> }
                    }).collect_view()
                }
            </div>
        }
        .into_any(),
        ProductionNode::Unresolved { item_id, amount } => view! {
            <div class="tree-node">
                 <div class="node-content node-missing">
                    <span class="connector">"x"</span>
                    {item_id} " x" {amount} " [MISSING RECIPE]"
                </div>
            </div>
        }
        .into_any(),
    }
}
