use clap::{Parser, Subcommand};
use rand::seq::IndexedRandom;
use rongta::TextSize;
use std::path::PathBuf;

#[derive(Debug, Subcommand)]
pub enum TemplateCommand {
    #[clap(about = "Create a box with random borders")]
    Box {
        #[clap(
            help = "The height of the box in rows. 1in ~= 8rows",
            default_value = "40"
        )]
        rows: Option<u32>,
    },
}

#[derive(Debug, Parser)]
pub struct TemplateArgs {
    #[clap(subcommand)]
    pub command: TemplateCommand,
}

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
    let printer = rongta::establish_rongta_printer()?;
    match args.command {
        TemplateCommand::Box { rows } => {
            let mut random = rand::rng();
            let templates = get_box_templates()?;
            let random_template = templates
                .choose(&mut random)
                .expect("This should have picked a random template");
            let mut print_builder = rongta::PrintBuilder::new();
            print_builder.cut = !no_cut;
            print_builder.add_content_no_check(
                &random_template.top,
                TextSize::Medium,
                true,
                false,
            )?;
            for _ in 0..rows.expect("We provided a default") {
                print_builder.add_content_no_check(
                    &random_template.row,
                    TextSize::Medium,
                    true,
                    false,
                )?;
            }
            print_builder.add_content_no_check(
                &random_template.bottom,
                TextSize::Medium,
                true,
                false,
            )?;
            print_builder.print(printer)?;
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
            18,
            templates.len(),
            "Either parsing logic error or the templates were updated to contain a different number of templates"
        );
        assert_eq!(
            ".----------------------------------------------.".to_string(),
            templates.first().unwrap().top
        );
        assert_eq!(
            "OOooOOooOOooOOooOOooOOooOOooOOooOOooOOooOOooOOoo".to_string(),
            templates.last().unwrap().bottom
        )
    }
}
