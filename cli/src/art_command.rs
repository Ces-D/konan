use clap::Parser;
use log::info;

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
