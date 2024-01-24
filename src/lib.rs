use std::{error, path::PathBuf};

use cli::Cli;
use clap::Parser;
mod cli;
pub mod task;

pub fn run() -> Result<(), Box<dyn error::Error>> {
    let cli = Cli::parse();

    let journey_file = cli.journal_file.unwrap_or(PathBuf::from("todo.json"));

    match cli.action {
        cli::Action::Add {task} => task::Task::add(journey_file, task)?,
        cli::Action::Remove { index } => task::Task::remove(journey_file, index)?,
        cli::Action::List => task::Task::list(journey_file)?,
    }

    Ok(())
}