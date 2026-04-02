use crate::clap_enum::DateBanner;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxTemplate {
    pub cut: bool,
    pub rows: Option<u32>,
    pub lined: bool,
    pub banner: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

impl From<BoxTemplatePulseRecipe> for BoxTemplate {
    fn from(value: BoxTemplatePulseRecipe) -> Self {
        Self {
            cut: value.cut,
            rows: value.rows,
            lined: value.lined,
            banner: value.banner,
            date: value.date.map(|v| v.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxTemplatePulseRecipe {
    pub cut: bool,
    pub rows: Option<u32>,
    pub lined: bool,
    pub banner: Option<String>,
    pub date: Option<DateBanner>,
}
