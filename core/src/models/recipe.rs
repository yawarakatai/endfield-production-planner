use crate::constants::SELF_REFERENCE_KEYWORD;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub by: String,
    pub time: u32,
    out: Option<u32>,
    #[serde(default)]
    pub inputs: HashMap<String, u32>,
    #[serde(default)]
    pub outputs: HashMap<String, u32>,
    #[serde(default)]
    pub is_source: bool,
}

impl Recipe {
    #[cfg(test)]
    pub fn new_for_test(
        id: String,
        by: String,
        time: u32,
        inputs: HashMap<String, u32>,
        outputs: HashMap<String, u32>,
        is_source: bool,
    ) -> Self {
        Recipe {
            id,
            by,
            time,
            out: None,
            inputs,
            outputs,
            is_source,
        }
    }

    pub fn normalize(&mut self) {
        if let Some(count) = self.out {
            self.outputs.insert(self.id.clone(), count);
        }

        if let Some(count) = self.outputs.remove(SELF_REFERENCE_KEYWORD) {
            self.outputs.insert(self.id.clone(), count);
        }
    }

    pub fn compute_unique_id(&self) -> String {
        let mut sorted_inputs: Vec<_> = self.inputs.iter().collect();
        sorted_inputs.sort_by_key(|(k, _)| *k);

        let inputs_str: String = sorted_inputs
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(",");

        format!("{}@{}[{}]", self.id, self.by, inputs_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_out_field() {
        // carbon from jincao has out=2
        let mut recipe = Recipe {
            id: "carbon".to_string(),
            by: "refining_unit".to_string(),
            time: 2,
            out: Some(2),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            is_source: false,
        };

        recipe.normalize();

        assert_eq!(recipe.outputs.get("carbon"), Some(&2));
        assert_eq!(recipe.outputs.len(), 1);
    }

    #[test]
    fn test_normalize_this_keyword() {
        // Test "this" keyword replacement
        let mut recipe = Recipe {
            id: "origocrust".to_string(),
            by: "refining_unit".to_string(),
            time: 2,
            out: None,
            inputs: HashMap::new(),
            outputs: vec![("this".to_string(), 1)]
                .into_iter()
                .collect(),
            is_source: false,
        };

        recipe.normalize();

        assert_eq!(recipe.outputs.get("origocrust"), Some(&1));
        assert_eq!(recipe.outputs.get("this"), None);
        assert_eq!(recipe.outputs.len(), 1);
    }

    #[test]
    fn test_compute_unique_id_deterministic() {
        // amethyst_component recipe with multiple inputs
        let recipe1 = Recipe {
            id: "amethyst_component".to_string(),
            by: "gearing_unit".to_string(),
            time: 10,
            out: None,
            inputs: vec![
                ("origocrust".to_string(), 5),
                ("amethyst_fiber".to_string(), 5),
            ]
            .into_iter()
            .collect(),
            outputs: HashMap::new(),
            is_source: false,
        };

        // Same recipe with inputs in different order
        let recipe2 = Recipe {
            id: "amethyst_component".to_string(),
            by: "gearing_unit".to_string(),
            time: 10,
            out: None,
            inputs: vec![
                ("amethyst_fiber".to_string(), 5),
                ("origocrust".to_string(), 5),
            ]
            .into_iter()
            .collect(),
            outputs: HashMap::new(),
            is_source: false,
        };

        let id1 = recipe1.compute_unique_id();
        let id2 = recipe2.compute_unique_id();

        assert_eq!(id1, id2);
        assert_eq!(id1, "amethyst_component@gearing_unit[amethyst_fiber:5,origocrust:5]");
    }
}
