use clap::{App, Arg, SubCommand, AppSettings};
use std::process::exit;
use std::env;
use kvs::{KvStore, Result};


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
        _ => {}
    }

    //    
    let path = env::current_dir().unwrap();
    let mut store = KvStore::open(path.as_path()).unwrap();
  
    if let Some(ref _matches) = matches.subcommand_matches("get") {
        let key = _matches.args["key"].vals[0].clone().into_string().unwrap();
        match store.get(key.to_owned()) {
            Some(val) => println!("{}", val),
            None => println!("Key not found")
        }
    }
    if let Some(ref _matches) = matches.subcommand_matches("rm") {
        let key = _matches.args["key"].vals[0].clone().into_string().unwrap();
        match store.get(key.to_owned()) {
            Some(val) => store.remove(key.to_owned()),
            None => {
                println!("Key not found");
                exit(1);
            }
        }
    }
    if let Some(ref _matches) = matches.subcommand_matches("set") {
    }
}