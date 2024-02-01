use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Website {
    pub name: String,
    pub about: String,
    pub url: String,
    pub owner: String,
}