use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Machine {
    pub id: String,
    pub tier: u32,
    pub power: u32,
}
