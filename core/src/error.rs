use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ProductionError {
    FileNotFound(String),
    ParseError(String),
    RecipeNotFound(String),
}

impl fmt::Display for ProductionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductionError::FileNotFound(path) => write!(f, "File not found: {}", path),
            ProductionError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ProductionError::RecipeNotFound(id) => write!(f, "Recipe not found: {}", id),
        }
    }
}

impl Error for ProductionError {}
