use clap::{Parser, Subcommand};
use log::info;
use rongta::{FormatState, TextDecoration};

#[derive(Debug, Subcommand)]
pub enum ArtCommand {
    #[clap(about = "Ask AI to draw something")]
    Draw {
        #[clap(help = "The idea for the art")]
        idea: String,
        #[clap(
            help = "The OpenAI model to use",
            short = 'm',
            default_value = "gpt-5.2-2025-12-11"
        )]
        model: Option<String>,
    },
    #[clap(about = "Create a banner")]
    Banner {
        #[clap(help = "The message")]
        message: String,
    },
}

#[derive(Debug, Parser)]
pub struct ArtArgs {
    #[clap(subcommand)]
    pub command: ArtCommand,
}

pub async fn handle_art_command(args: ArtArgs, cut: bool) -> anyhow::Result<()> {
    match args.command {
        ArtCommand::Draw { idea, model } => {
            let response =
                ai::generate_ascii_art(&idea, &model.expect("Provide a default model")).await?;
            info!("Response from OpenAI: {}", response);
            let mut builder = rongta::PrintBuilder::new(cut);
            for c in response.chars() {
                builder.add_char_content(rongta::StyledChar {
                    ch: c,
                    state: FormatState {
                        text_size: rongta::TextSize::Medium,
                        text_decoration: TextDecoration {
                            bold: true,
                            ..Default::default()
                        },
                    },
                })?;
            }
            builder.print(None)?;
            Ok(())
        }
        ArtCommand::Banner { message } => {
            let pattern = crate::sytem_design::get_random_box_pattern()?;
            let mut builder = rongta::PrintBuilder::new(cut);
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.new_line();
            for c in message.chars() {
                builder.add_char_content(rongta::StyledChar {
                    ch: c,
                    state: FormatState {
                        text_size: rongta::TextSize::ExtraLarge,
                        text_decoration: rongta::TextDecoration {
                            bold: true,
                            ..Default::default()
                        },
                    },
                })?;
            }
            builder.new_line();
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.new_line();
            builder.print(None)?;
            Ok(())
        }
    }
}
