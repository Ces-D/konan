mod tools;

use clap::{
    builder::NonEmptyStringValueParser, crate_authors, crate_description, crate_name,
    crate_version, Arg, ArgAction, Command,
};
use log::{error, info};
use rongta::{establish_rongta_printer, PrintBuilder, TextSize};

fn main() {
    env_logger::builder().init();

    let cli = Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::new("content")
                .action(ArgAction::Append)
                .required(true)
                .help("Content that will be printed")
                .value_parser(NonEmptyStringValueParser::new()),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .action(ArgAction::SetTrue)
                .help("A flag identifying the content as a file path"),
        )
        .arg(
            Arg::new("historic")
                .long("historic")
                .short('h')
                .action(ArgAction::SetTrue)
                .help("A flag for historic file printing. Only used when file flag used."),
        )
        .arg(
            Arg::new("no-cut")
                .long("no-cut")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("A flag identifying that the printer should not cut after printing"),
        )
        .arg(
            Arg::new("bold")
                .long("bold")
                .short('b')
                .action(ArgAction::SetTrue)
                .help("A flag identifying this content as BOLD"),
        )
        .arg(
            Arg::new("underline")
                .long("underline")
                .short('u')
                .action(ArgAction::SetTrue)
                .help("A flag identifying this content as UNDERLINED"),
        )
        .arg(
            Arg::new("text")
                .long("text")
                .short('t')
                .action(ArgAction::Set)
                .value_parser(["normal", "large", "x-large"])
                .default_value("normal")
                .help("The text size for this content"),
        );

    let args = cli.get_matches();
    let content: Vec<String> = args
        .get_many("content")
        .ok_or_else(|| {
            vec!["Nothing to print. Future version will be return a joke from chatjippity for empty content".to_string()]
        }).unwrap().cloned().collect();
    let is_file = args.get_flag("file");
    let print_file_historic = args.get_flag("historic");
    let cut = !args.get_flag("no-cut");
    let format_bold = args.get_flag("bold");
    let format_underline = args.get_flag("underline");
    let text_size = match args.get_one::<String>("text").map(|s| s.as_str()) {
        Some("normal") => TextSize::Medium,
        Some("large") => TextSize::Large,
        Some("x-large") => TextSize::ExtraLarge,
        _ => TextSize::Medium,
    };

    let mut print_builder = PrintBuilder::new();
    print_builder.cut = cut;

    if is_file {
        info!("Ignoring `bold`, `underline`, and `text_size` flags");
        let file_path = content.first().expect("Failed to interpret the file path");
        if print_file_historic {
            let file_content =
                tools::file::read_file_complete(file_path).expect("Failed to read file path");
            print_historic(&file_content);
        } else {
            let file = tools::file::read_file_lines(file_path).expect("Failed to read file lines");
            for line in file {
                if let Ok(c) = line {
                    print_builder
                        .add_content(&c, text_size, false, false)
                        .expect("Failed to add file content");
                }
            }
            print(print_builder)
        }
    } else {
        for c in content.iter() {
            print_builder
                .add_content(&c, text_size, format_bold, format_underline)
                .expect("Failed to add content");
        }
        print(print_builder)
    }
}

fn print(builder: PrintBuilder) {
    match establish_rongta_printer() {
        Ok(printer) => match builder.print(printer) {
            Ok(_) => info!("Succesfully printed!"),
            Err(_) => error!("Failed to print!"),
        },
        Err(_) => error!("Unable to connect to rongta printer"),
    }
}

fn print_historic(content: &str) {
    match establish_rongta_printer() {
        Ok(printer) => match PrintBuilder::print_historic(content, printer) {
            Ok(_) => info!("Succesfully printed!"),
            Err(_) => error!("Failed to print!"),
        },
        Err(_) => error!("Unable to connect to rongta printer"),
    }
}
