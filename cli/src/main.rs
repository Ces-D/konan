use clap::{
    Arg, ArgAction, Command, builder::NonEmptyStringValueParser, crate_authors, crate_description,
    crate_name, crate_version,
};

fn main() {
    let cli = Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(
            Arg::new("content")
                .action(ArgAction::Set)
                .required(true)
                .help("Content that will be printed")
                .value_parser(NonEmptyStringValueParser::new()),
        )
        .arg(
            Arg::new("link")
                .long("link")
                .short('l')
                .action(ArgAction::SetTrue),
        );
    let args = cli.get_matches();
    let content: String = args
        .get_one::<String>("content")
        .map(|v: &String| v.clone())
        .unwrap_or_else(|| {
            "Nothing to print. Future version will be a joke from chatjippity".to_string()
        });
    let is_link = args.get_flag("link");

    if is_link {
        todo!()
    } else {
        match rongta::establish_rongta_printer() {
            Ok(printer) => match rongta::print(content, printer) {
                Ok(_) => println!("Succesfully printed"),
                Err(e) => eprintln!("{}", e),
            },
            Err(e) => println!("{}", e),
        };
    }
}
