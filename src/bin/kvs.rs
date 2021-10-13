use clap::{App, Arg, SubCommand, AppSettings};
use std::process::exit;


fn main() {
    let matches = App::new("Simple key-value store")
                            .setting(AppSettings::ArgRequiredElseHelp)
                            .version("1.0")
                            .author("Dai D. <ddao@ualberta.ca>")
                            .about("Does awesome things")
                            .arg(Arg::with_name("V")
                                .short("V")
                                .multiple(true)
                                .help("Sets the level of verbosity"))
                            .subcommand(SubCommand::with_name("get")
                                        .about("Get string value of a given string key")
                                        .arg(Arg::with_name("key")
                                                .help("String value of key")
                                                .required(true)))
                            .subcommand(SubCommand::with_name("set")
                                        .about("Set string value of a given string key and value")
                                        .arg(Arg::with_name("key")
                                                .help("String value of key")
                                                .required(true))
                                        .arg(Arg::with_name("value")
                                                .help("String value of value")
                                                .required(true)))
                            .subcommand(SubCommand::with_name("rm")
                                        .about("Remove string value of a given string key")
                                        .arg(Arg::with_name("key")
                                                .help("String value of key")
                                                .required(true)))
                          .get_matches();

    match matches.occurrences_of("V") {
        1 => println!(env!("CARGO_PKG_VERSION")),
        _ => println!("No version arg"),
    }
    
    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level app
    if let Some(ref _matches) = matches.subcommand_matches("get") {
        eprintln!("unimplemented");
        exit(1);
    }
    if let Some(ref _matches) = matches.subcommand_matches("rm") {
        eprintln!("unimplemented");
        exit(1);
    }
    if let Some(ref _matches) = matches.subcommand_matches("set") {
        eprintln!("unimplemented");
        exit(1);
    }
}