use anyhow::Context;
use chrono::{Duration, Local};
use clap::{Parser, Subcommand};
use log::trace;
use rand::seq::IndexedRandom;
use rongta::{Justify, PrintBuilder, TextDecoration};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum Heading {
    Today,
    Tomorrow,
}

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[clap(about = "Create a box with random borders")]
    Box {
        #[clap(
            help = "The height of the box in rows. 1in ~= 8rows",
            default_value = "28"
        )]
        rows: Option<u32>,
        #[clap(short, long, help = "Something to print on the top of the template")]
        banner: Option<Heading>,
    },
}

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[clap(subcommand)]
    pub command: TemplateCommand,
}

#[derive(Clone)]
struct BoxTemplate {
    top: String,
    row: String,
    bottom: String,
}

fn get_konan_templates() -> PathBuf {
    PathBuf::from(
        std::env::var("KONAN_TEMPLATES")
            .expect("KONAN_TEMPLATES env var pointing to templates should be in PATH"),
    )
}

fn get_box_templates() -> anyhow::Result<Vec<BoxTemplate>> {
    let patterns_path = get_konan_templates().join("box_patterns.txt");
    let content = std::fs::read_to_string(patterns_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let templates = lines
        .chunks(4) // Each pattern is 3 lines + 1 empty separator
        .filter_map(|chunk| {
            if chunk.len() >= 3 {
                Some(BoxTemplate {
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

pub async fn handle_template_command(args: TemplateArgs, no_cut: bool) -> anyhow::Result<()> {
    match args.command {
        TemplateCommand::Box { rows, banner } => {
            let mut random = rand::rng();
            let templates = get_box_templates()?;
            let random_template = templates
                .choose(&mut random)
                .with_context(|| "Failed to choose a random template")?;
            trace!("Template top:    {:?}", random_template.top);
            trace!("Template row:    {:?}", random_template.row);
            trace!("Template bottom: {:?}", random_template.bottom);
            let mut builder = PrintBuilder::new(!no_cut);

            if let Some(banner) = banner {
                builder.set_text_decoration(TextDecoration {
                    bold: true,
                    ..Default::default()
                });
                builder.add_content(&random_template.top)?;

                let date = match banner {
                    Heading::Today => Local::now(),
                    Heading::Tomorrow => Local::now() + Duration::days(1),
                };
                let date_str = date.format("%A, %B %d, %Y").to_string();

                builder.set_justify_content(Justify::Center);
                builder.add_content(&date_str)?;
            }
            builder.set_justify_content(Justify::Left);
            builder.set_text_decoration(TextDecoration {
                bold: true,
                ..Default::default()
            });

            builder.add_content(&random_template.top)?;
            for _ in 0..rows.expect("We provided a default") {
                builder.add_content(&random_template.row)?;
            }
            builder.add_content(&random_template.bottom)?;
            builder.print()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_all_box_templates() {
        let templates = get_box_templates().expect("Failed to read box templates file");
        assert_eq!(
            23,
            templates.len(),
            "Either parsing logic error or the templates were updated to contain a different number of templates"
        );
        assert_eq!(
            ".----------------------------------------------.".to_string(),
            templates.first().unwrap().top
        );
        assert_eq!(
            "&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&&".to_string(),
            templates.last().unwrap().bottom
        )
    }
}
