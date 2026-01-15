use web_sys::{wasm_bindgen, window};

/// URL parameters for production planning.
#[derive(Debug, Clone, Default)]
pub struct UrlParams {
    pub item: Option<String>,
    pub amount: Option<u32>,
}

/// Parses URL parameters from the current browser URL.
pub fn parse_url_params() -> UrlParams {
    let mut params = UrlParams::default();

    let Some(window) = window() else {
        return params;
    };

    let Ok(location) = window.location().href() else {
        return params;
    };

    let Ok(url) = web_sys::Url::new(&location) else {
        return params;
    };
    let search_params = url.search_params();

    if let Some(item) = search_params.get("item") {
        if !item.is_empty() {
            params.item = Some(item);
        }
    }

    if let Some(amount_str) = search_params.get("amount") {
        if let Ok(amount) = amount_str.parse::<u32>() {
            if amount > 0 {
                params.amount = Some(amount);
            }
        }
    }

    params
}

/// Updates the browser URL with the given parameters without reloading.
/// Uses History API's replaceState to update URL silently.
pub fn update_url_params(item: &str, amount: u32) {
    let Some(window) = window() else {
        return;
    };

    let Ok(location) = window.location().href() else {
        return;
    };

    let Ok(url) = web_sys::Url::new(&location) else {
        return;
    };

    let search_params = url.search_params();
    search_params.set("item", item);
    search_params.set("amount", &amount.to_string());

    let new_url = format!("{}?{}", url.pathname(), search_params.to_string());

    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&new_url));
    }
}

/// Generates a shareable URL string for the given parameters.
pub fn generate_share_url(item: &str, amount: u32) -> Option<String> {
    let window = window()?;
    let location = window.location().href().ok()?;
    let url = web_sys::Url::new(&location).ok()?;

    let search_params = url.search_params();
    search_params.set("item", item);
    search_params.set("amount", &amount.to_string());

    Some(format!(
        "{}//{}{}?{}",
        url.protocol(),
        url.host(),
        url.pathname(),
        search_params.to_string()
    ))
}
