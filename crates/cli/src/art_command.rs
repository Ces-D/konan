use clap::{Parser, Subcommand};
use rongta::{
    RongtaPrinter, SupportedDriver,
    elements::{FormatState, StyledChar, TextSize},
};

#[derive(Debug, Subcommand)]
pub enum ArtCommand {
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
        ArtCommand::Banner { message } => {
            let pattern = designs::get_random_box_pattern()?;
            let mut builder = RongtaPrinter::new(cut);
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.new_line();
            for c in message.chars() {
                builder.add_char_content(StyledChar {
                    ch: c,
                    state: FormatState {
                        text_size: TextSize::ExtraLarge,
                        is_bold: true,
                    },
                })?;
            }
            builder.new_line();
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.add_content(&pattern.top)?;
            builder.new_line();
            builder.new_line();
            builder.print(None, SupportedDriver::Console)?;
            Ok(())
        }
    }
}
