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
    // the tags fields can be modelled either way. If modelled with 
    // Vec<String> then an empty Vec represents a task with no tag.
    // If modelled with Option<Vec<String>>, then there seems to be two
    // representations for a task with no tag, but it might be more 
    // memory efficient if the None case is always utilised for representation
    // tags: Vec<String>,
    tags: Option<Vec<String>>,
    #[serde(with = "ts_seconds")]
    creted_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum State {
    Active,
    Complete,
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
    fn new(task_name: String, task_tags: Option<Vec<String>>) -> Self {
        Task {
            name: task_name,
            state: State::Active,
            tags: task_tags,
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
    /// task::Task::add(PathBuf::from("todo.json"), "play".to_string(), Some(vec!["good first issue".to_string()]));
    /// ```
    pub fn add(journal_path: PathBuf, name: String, tags: Option<Vec<String>>) -> ioResult<()> {
        let f = OpenOptions::new()
                                .write(true)
                                .create(true)
                                .read(true)
                                .open(&journal_path)?;

        let f = BufReader::new(f);

        let mut tasks = Self::_get_tasks(f)?;

        let new_task = Self::new(name, tags);

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

    /// The method fetches the current tasks into a vec from the Json
    /// and prints them out. Empty tasks is specifically handled within
    /// # Examples
    /// ```
    /// use rusty_journal_clap::task;
    /// use std::path::PathBuf;
    /// task::Task::list(PathBuf::from("todo.json"));
    /// ```    
    pub fn list(journal_path: PathBuf, tag: Option<&String>) -> ioResult<()> {        
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
            match tag {
                Some(tag) => {
                    for task in tasks.iter()
                                            .filter(|&task| task.tags.as_ref()
                                                // the family of mapping methods (e.g. map, is_some_and) on Option type would consume the ownership of the Option
                                                // here the task.tags is a field of the Task struct, of Option<Vec<String>> type
                                                // If directly followed by a is_some_and call, the ownership of the field would move out of the Task struct
                                                // which obviously is a violation as it wouldn't be allowed by the compiler either
                                                // The as_ref method of Option type is handy here since it creates another owned Option instance to be CONSUMED
                                                // plus with the same refereced data inside the Option for further ops 
                                                .is_some_and(|tags| tags
                                                    .contains(&tag))) {
                                                        println!("{}", task);
                                                    }
                         
                },
                None => {
                    for task in tasks {
                        println!("{}", task);
                    }
                }
            }
        }

        Ok(())
    }    

    /// This method helps with testing by clearing all the data
    /// # Examples:
    /// ```
    /// use rusty_journal_clap::task;
    /// use std::path::PathBuf;
    /// task::Task::clear(PathBuf::from("todo.json"));
    /// ```
    pub fn clear(journal_path: PathBuf) -> ioResult<()> {
        let f = OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .read(true)
                                    .open(&journal_path)?;

        let f = BufReader::new(f);

        let mut tasks = Self::_get_tasks(f)?;

        tasks.clear();

        let f = OpenOptions::new()
                            .truncate(true)
                            .write(true)
                            .open(&journal_path)?;

        let f = BufWriter::new(f); 
        Self::_write_tasks(&tasks, f)?;

        Ok(())        
    }


}