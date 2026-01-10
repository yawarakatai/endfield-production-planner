use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct Recipe {
    pub id: String,
    pub by: Vec<String>,
    pub time: u32,
    #[serde(default)]
    pub inputs: HashMap<String, u32>,
    #[serde(default)]
    pub outputs: HashMap<String, u32>,
}
