use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KonanFile {
    pub cut: bool,
    pub name: String,
    pub rows: Option<u32>,
}
