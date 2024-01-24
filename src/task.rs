use std::{path::PathBuf, fs::OpenOptions, io::{BufReader, Read, Write, BufWriter, Error, ErrorKind}};
use std::io::Result as ioResult;
use chrono::{DateTime, Utc, serde::ts_seconds, Local};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt::{Display, Formatter};
use std::fmt::Result as fmtResult;

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    name: String,
    state: State,
    tags: Vec<Tag>,
    notes: Option<String>,
    #[serde(with = "ts_seconds")]
    creted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum State {
    Active,
    Complete,
}

#[derive(Debug, Deserialize, Serialize)]
struct Tag {
    tag_str: String,
}

impl Display for Task {
    fn fmt (&self, f: &mut Formatter) -> fmtResult {
        // format syntax c.f.
        // learn.microsoft.com/en-us/training/modules/rust-create-command-line-program/7-list-tasks-function
        // https://doc.rust-lang.org/std/fmt/index.html#fillalignment
        write!(f, "Task: {:<50} Created at: {}", self.name, self.creted_at.with_timezone(&Local).format("%d/%m/%Y %H:%M"))
    }
}

impl Task {
    fn new(name: String) -> Self {
        Task {
            name,
            state: State::Active,
            tags: Vec::new(),
            notes: None,
            creted_at: Utc::now(),
        }
    }

    fn _get_tasks(file: impl Read) -> ioResult<Vec<Task>>  {
        let tasks = match serde_json::from_reader(file)  {
            Ok(tasks) => tasks,
            Err(err) if err.is_eof() => Vec::new(),
            Err(err) => Err(err)?,
        };

        Ok(tasks)
    }

    fn _write_tasks(tasks: &Vec<Task>, file: impl Write) -> ioResult<()> {
        serde_json::to_writer(file, tasks)?;
        Ok(())
    }


    /// The method fetches the current tasks as a vec from the Json
    /// and add a new task by pushing to the vec and write back to the Json
    /// # Examples
    /// ```
    /// use rusty_journal_clap::task;
    /// use std::path::PathBuf;
    /// task::Task::add(PathBuf::from("todo.json"), "new_task".to_string());
    /// ```
    pub fn add(journal_path: PathBuf, name: String) -> ioResult<()> {
        let f = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .read(true)
                                .open(&journal_path)?;

        let f = BufReader::new(f);

        let mut tasks = Self::_get_tasks(f)?;

        let new_task = Self::new(name);

        tasks.push(new_task);

        let f = OpenOptions::new()
                            // technically not stricted needed as overwritten data 
                            // is larger than what was in the file at this point of the add operation
                            .truncate(true) 
                            .write(true)
                            .open(&journal_path)?;

        let f = BufWriter::new(f);

        Self::_write_tasks(&tasks, f)?;

        Ok(())
    }

    /// The method fetches the current tasks into a vec from the Json
    /// and prints them out. Empty tasks is specifically handled within
    /// # Examples
    /// ```
    /// use rusty_journal_clap::task;
    /// use std::path::PathBuf;
    /// task::Task::list(PathBuf::from("todo.json"));
    /// ```    
    pub fn list(journal_path: PathBuf) -> ioResult<()> {        
        let f = OpenOptions::new()
                            // write must be set first to enable create
                            // https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create
                            .write(true)
                            .create(true)
                            .read(true)
                            .open(&journal_path)?;


        let f = BufReader::new(f);

        let tasks = Self::_get_tasks(f)?;

        if tasks.is_empty() {
            println!("Empty to-do list");
        } else {
            for task in tasks {
                println!("{}", task);
            }
        }

        Ok(())
    }

    /// The method fetches the current tasks into a vec from the Json
    /// and prints them out. Empty tasks is specifically handled within
    /// # Examples
    /// ```
    /// use rusty_journal_clap::task;
    /// use std::path::PathBuf;
    /// task::Task::remove(PathBuf::from("todo.json"), 1);
    /// ```      
    pub fn remove(journal_path: PathBuf, index: usize) -> ioResult<()> {
        let f = OpenOptions::new()
                            .read(true)
                            .open(&journal_path)?;
        
        let f = BufReader::new(f);

        let mut tasks = Self::_get_tasks(f)?;


        // Thinking from the user input perspective:
        // User is expected to put in an index from 0 to the number of tasks (task.len())
        // Hence that expectaion is combined with index bound check and error reporting as following
        if index == 0 || index > tasks.len() {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid Task ID"));

        }
        // With the check above in place, this remove call is certain to NOT PANIC
        tasks.remove(index-1);


        let f = OpenOptions::new()
                    // Required as otherwise we overwrite the file with data smaller than
                    // the current content presumably from the start of file as seek position
                    // which corrupts the data
                    .truncate(true)
                    .write(true)
                    .open(&journal_path)?;

        let f = BufWriter::new(f);

        Self::_write_tasks(&tasks, f)?;

        Ok(())
    }


}