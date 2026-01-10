use crate::constants::SELF_REFERENCE_KEYWORD;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub by: Vec<String>,
    pub time: u32,
    out: Option<u32>,
    #[serde(default)]
    pub inputs: HashMap<String, u32>,
    #[serde(default)]
    pub outputs: HashMap<String, u32>,
}

impl Recipe {
    pub fn normalize(&mut self) {
        if let Some(count) = self.out {
            self.outputs.insert(self.id.clone(), count);
        }

        if let Some(count) = self.outputs.remove(SELF_REFERENCE_KEYWORD) {
            self.outputs.insert(self.id.clone(), count);
        }
    }
}
