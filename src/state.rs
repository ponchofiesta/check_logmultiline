/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Load and save log file states.

use serde::{Serialize, Deserialize};

/// Holds the state informations about a log file.
#[derive(Serialize, Deserialize)]
pub struct State {

    /// Path to the log file.
    pub path: std::path::PathBuf,

    /// Size of the log file.
    pub size: u64,

    /// Creation date of the log file.
    pub created: std::time::SystemTime,

    /// Last analyzed line number of the log file.
    pub line_number: i64,
}

/// A state document holding several log file states.
#[derive(Serialize, Deserialize)]
pub struct StateDoc {

    /// A list of log file states.
    pub states: Vec<State>
}

impl StateDoc {

    /// Create a new default state document.
    pub fn new() -> Self {
        StateDoc {
            states: vec![]
        }
    }
}

/// Save or load a log file state to or from file.
pub struct StateLoader {

    /// The path to the state file.
    file: std::path::PathBuf,
}

impl State {

    /// Create a new default log file state.
    pub fn new(log_file: std::path::PathBuf) -> Self {
        State {
            path: log_file,
            size: 0,
            created: std::time::SystemTime::UNIX_EPOCH,
            line_number: -1,
        }
    }
}

impl StateLoader {

    /// Create a new default instance.
    /// # Arguments
    /// * `path` - Path to the state file
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        StateLoader {
            file: path.as_ref().to_path_buf(),
        }
    }

    /// Load a state document from a file.
    pub fn load(&self) -> Result<StateDoc, String> {
        if self.file.is_dir() {
            return Err(String::from("Statefile is a directory"));
        }
        if !self.file.exists() {
            return Ok(StateDoc::new())
        }
        match std::fs::read_to_string(&self.file) {
            Ok(states_content) => match serde_json::from_str(&states_content) {
                Ok(states) => Ok(states),
                Err(e) => Err(format!("Could not parse statefile: {}", e))
            },
            _ => Err(String::from("Could not read statefile"))
        }
    }

    /// Save the state to a state file.
    /// # Arguments
    /// * `state` - State to be saved
    pub fn save(&self, state: &StateDoc) -> Result<(), String> {
        let content = match serde_json::to_string_pretty(state) {
            Ok(content) => content,
            Err(err) => return Err(format!("Could not encode statefile: {}", err))
        };
        match std::fs::write(&self.file, content) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Could not save statefile: {}", err))
        }
    }
}