use clap::Parser;

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
    let mut printer = rongta::establish_rongta_printer()?;
    if no_cut {
        printer.writeln(&response)?.print()?;
    } else {
        printer.writeln(&response)?.print_cut()?;
    }
    Ok(())
}
