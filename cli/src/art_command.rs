use clap::Parser;
use log::info;
use rongta::{FormatState, StyledChar, TextDecoration};

#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
enum FontSize {
    ExtraLarge,
    #[default]
    Large,
}

#[derive(Debug, Parser)]
pub struct ArtArgs {
    #[clap(help = "The idea for the art")]
    idea: String,
    #[clap(
        help = "The OpenAI model to use",
        short = 'm',
        default_value = "gpt-5-nano-2025-08-07"
    )]
    model: Option<String>,
}

#[derive(Debug, Parser)]
pub struct BigTextArgs {
    #[clap(help = "The text to print in large format")]
    text: String,
    #[clap(help = "Font size", short = 's', default_value = "large")]
    size: Option<FontSize>,
}

pub async fn handle_art_command(args: ArtArgs, no_cut: bool) -> anyhow::Result<()> {
    let response = ai::generate_ascii_art(
        &args.idea,
        &args.model.expect("We provided a default value"),
    )
    .await?;
    info!("Response from OpenAI: {}", response);
    let mut printer = rongta::establish_rongta_printer()?;
    printer.bold(true)?;
    printer.writeln(&response)?;
    match !no_cut {
        true => printer.print_cut()?,
        false => printer.print()?,
    };
    Ok(())
}

pub async fn handle_big_text_command(args: BigTextArgs, no_cut: bool) -> anyhow::Result<()> {
    let size = match args.size.unwrap_or_default() {
        FontSize::ExtraLarge => rongta::TextSize::ExtraLarge,
        FontSize::Large => rongta::TextSize::Large,
    };

    let mut builder = rongta::PrintBuilder::new(!no_cut);
    builder.set_justify_content(rongta::Justify::Center);

    for char in args.text.chars() {
        builder.add_char_content(StyledChar {
            ch: char,
            state: FormatState {
                text_size: size,
                text_decoration: TextDecoration {
                    bold: true,
                    underline: true,
                    ..Default::default()
                },
            },
        })?;
    }

    builder.print()?;
    Ok(())
}
