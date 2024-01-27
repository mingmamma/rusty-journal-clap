use std::{error, path::PathBuf};
use clap::{value_parser, Arg, ArgAction, Command};
mod cli;
pub mod task;

pub fn run() -> Result<(), Box<dyn error::Error>> {
    let arg_matches = Command::new("My Program")
    .author("Me, me@mail.com")
    .version("1.0.2")
    .about("Explains in brief what the program does")
    .arg(
        Arg::new("journal_file")
        .long("journal_file")
        // used to user-facing help msg: https://docs.rs/clap/latest/clap/struct.Arg.html#method.value_name
        .value_name("FILE")
        .default_value("todo.json")
        // used to type-check user input: https://docs.rs/clap/latest/clap/struct.Arg.html#method.value_parser
        .value_parser(value_parser!(PathBuf))
    )
    .subcommand(
Command::new("add")
            .arg(Arg::new("task")
                    .long("task")
                    .required(true)
            )
            .arg(Arg::new("tag")
                    .action(ArgAction::Append)
                    .long("tag")
            )
    )
    .subcommand(
Command::new("remove")
            .arg(Arg::new("index")
                    .required(true)
                    .value_parser(value_parser!(usize))
            )
    )
    .subcommand(
Command::new("list")
            .arg(Arg::new("tag")
                    .value_parser(value_parser!(String))
                    .long("tag")                    
            )    
    )    
    .after_help("Longer explanation to appear after the options when \
                 displaying the help information from --help or -h")
    .get_matches();

    let journal_file = arg_matches.get_one::<PathBuf>("journal_file").unwrap().to_owned();
    
    // Comment: the following block of code works by destructuring the subcommand of the arg_matches struct
    // Currently, in every destructuring instance, the desirable arg is extracted from the args_matches struct
    // with to_owned() call to create an owned instance. There could be more fine-grained case-by-case consideration
    // since some desirable arg are needed as owned instance to further be consumed e.g. adding new tasks would need to consume
    // owned attributes in the creation of new owned Task instance anyway
    // However, ops like removal by index or listing by tag DOES NOT SEEM TO neccessarily need to own the arg
    // since they just need to reference the info from the arg to complete their jobs. The removal of to_owned call
    // in this cases SEEM TO be benefitial w.r.t performance
    match arg_matches.subcommand() {
        Some(("list", list_args)) => {
            let list_tag = list_args.get_one::<String>("tag")
                                                     .to_owned();
            task::Task::list(journal_file, list_tag)?
        },
        Some(("remove", remove_args)) => {
            let remove_index = remove_args.get_one::<usize>("index")
                                                 .unwrap()
                                                 .to_owned();
            task::Task::remove(journal_file, remove_index)?
        },
        Some(("add", add_args)) => {
            let add_task_name = add_args.get_one::<String>("task")
                                                .unwrap()
                                                .to_owned();

            let add_task_tags  = add_args.get_many::<String>("tag")
                                            // Since the get_many call returns an Option, to process the Some() case further and 
                                            // levae the None case as is, and_then() is used, refered as the "flatMap" equivalent
                                            .and_then(|x|
                                                // the Some() case is a ValuesRef struct which is an iterator resulting from
                                                // the get_many call. Thus just needing to processing it properly and collecting
                                                // into a collection: https://docs.rs/clap/latest/clap/parser/struct.ValuesRef.html
                                                Some(x.map(|s| 
                                                    s.to_owned()).collect::<Vec<_>>()));


            task::Task::add(journal_file, add_task_name, add_task_tags)?
        }
        _ => unreachable!(),
    }

    Ok(())
}