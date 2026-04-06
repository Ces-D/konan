use crate::clap_enum::AllowedCommand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KonanFile {
    pub cut: bool,
    pub name: String,
    pub prehook_command: Option<AllowedCommand>,
    pub prehook_command_arg: Option<String>,
    pub rows: Option<u32>,
}
