mod components;
mod utils;

use components::App;

fn main() {
    leptos::mount::mount_to_body(|| leptos::prelude::view! { <App/> })
}
