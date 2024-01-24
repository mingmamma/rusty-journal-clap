use std::path::PathBuf;
use clap::{Parser, Subcommand};


#[derive(Parser)]
#[command(version = "0.1.24", author = "Reggie Mantle", about = "Does really amazing things for great people")]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub journal_file: Option<PathBuf>,
    
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    /// Write tasks to the journal file.
    Add {
        task: String,
    },
    /// Remove an entry from the journal file by position.
    Remove {
        index: usize,
    },
    /// List all tasks in the journal file.
    List,
}