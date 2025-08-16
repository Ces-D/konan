mod tools;

use std::str::FromStr;

use clap::{
    Arg, ArgAction, Command,
    builder::{NonEmptyStringValueParser, PossibleValuesParser},
    crate_authors, crate_description, crate_name, crate_version, value_parser,
};
use rongta::{Template, TemplateVariation};
use strum::VariantNames;

fn main() {
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
            Arg::new("link")
                .long("link")
                .short('l')
                .action(ArgAction::SetTrue)
                .help("A flag identifying the content as a link"),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .short('f')
                .action(ArgAction::SetTrue)
                .help("A flag identifying the content as a file path"),
        )
        .arg(
            Arg::new("min_lines")
                .long("min_lines")
                .short('m')
                .action(ArgAction::Set)
                .default_value("0")
                .value_parser(value_parser!(u8))
                .help("Set the min number of lines to print"),
        )
        .arg(
            Arg::new("template")
                .long("template")
                .short('t')
                .action(ArgAction::Set)
                .default_value(TemplateVariation::Raw.as_ref())
                .value_parser(PossibleValuesParser::new(TemplateVariation::VARIANTS))
                .help("Templates add styles to the print"),
        );
    let args = cli.get_matches();
    let content: Vec<String> = args
        .get_many("content")
        .ok_or_else(|| {
            vec!["Nothing to print. Future version will be return a joke from chatjippity for empty content".to_string()]
        }).unwrap().cloned().collect();
    let is_link = args.get_flag("link");
    let is_file = args.get_flag("file");
    let template_variation = args
        .get_one::<String>("template")
        .map(|v| TemplateVariation::from_str(v.as_str()).unwrap())
        .unwrap_or_default()
        .clone();
    let min_lines = args.get_one::<u8>("min_lines").cloned().unwrap_or_default();

    if is_link {
        todo!()
    } else if is_file {
        let file_path = content.first().unwrap();
        let content = tools::file::read_file(file_path).expect("Failed to open file");
        print(Template {
            content: vec![content],
            min_lines,
            variation: template_variation,
        })
    } else {
        print(Template {
            content,
            min_lines,
            variation: template_variation,
        })
    }
}

fn print(content: Template) {
    match rongta::establish_rongta_printer() {
        Ok(printer) => match rongta::print(content, printer) {
            Ok(_) => println!("Succesfully printed"),
            Err(e) => eprintln!("{}", e),
        },
        Err(e) => println!("{}", e),
    };
}
