use resource_calculator_core::i18n::Localizer;
use std::collections::HashSet;

/// Helper function to get the localized name for an item ID.
/// Checks if the ID is a machine and uses the appropriate localizer method.
pub fn get_localized_name(
    item_id: &str,
    localizer: &Localizer,
    machine_ids: &HashSet<String>,
) -> String {
    if machine_ids.contains(item_id) {
        localizer.get_machine(item_id)
    } else {
        localizer.get_item(item_id)
    }
}
