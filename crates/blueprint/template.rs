pub mod box_outline;
pub mod habit_tracker;
use anyhow::Context;
use rand::seq::IndexedRandom;
use std::path::PathBuf;

fn konan_templates() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/template"))
}

fn get_box_patterns() -> anyhow::Result<Vec<BoxPattern>> {
    let patterns_path = konan_templates().join("box_patterns.txt");
    let content = std::fs::read_to_string(patterns_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let templates = lines
        .chunks(4) // Each pattern is 3 lines + 1 empty separator
        .filter_map(|chunk| {
            if chunk.len() >= 3 {
                Some(BoxPattern {
                    top: chunk[0].to_string(),
                    row: chunk[1].to_string(),
                    bottom: chunk[2].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(templates)
}

pub fn get_random_box_pattern() -> anyhow::Result<BoxPattern> {
    let mut random = rand::rng();
    let templates = get_box_patterns()?;
    let random_template = templates
        .choose(&mut random)
        .with_context(|| "Failed to choose a random template")?;
    log::trace!("Template top:    {:?}", random_template.top);
    log::trace!("Template row:    {:?}", random_template.row);
    log::trace!("Template bottom: {:?}", random_template.bottom);

    Ok(random_template.to_owned())
}

#[derive(Clone)]
pub struct BoxPattern {
    pub top: String,
    pub row: String,
    pub bottom: String,
}
