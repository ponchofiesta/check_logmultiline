/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Load and save log file states.

use crate::logfile::Match;
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Holds the state informations about a log file.
#[derive(Serialize, Deserialize)]
pub struct State {
    /// Path to the log file.
    pub path: PathBuf,

    /// Size of the log file.
    #[serde(default)]
    pub size: u64,

    /// Creation date of the log file.
    #[serde(default = "SystemTime::now")]
    pub created: SystemTime,

    /// Last analyzed line number of the log file.
    #[serde(default)]
    pub line_number: i64,

    /// Kept messages from previous runs
    #[serde(default)]
    pub kept_matches: Vec<Match>,
}

impl State {
    /// Create a new default log file state.
    pub fn new(log_file: PathBuf) -> Self {
        State {
            path: log_file,
            size: 0,
            created: SystemTime::UNIX_EPOCH,
            line_number: -1,
            kept_matches: vec![],
        }
    }
}

/// A state document holding several log file states.
#[derive(Serialize, Deserialize)]
pub struct StateDoc {
    /// A list of log file states.
    #[serde(default)]
    pub states: Vec<State>,
}

/// Save or load a log file state to or from file.
pub struct StateLoader {
    /// The path to the state file.
    path: PathBuf,

    /// The file handle to the state file.
    file: Option<File>,
}

impl StateLoader {
    /// Create a new default instance.
    /// # Arguments
    /// * `path` - Path to the state file
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        StateLoader {
            path: path.as_ref().to_path_buf(),
            file: None,
        }
    }

    /// Load a state document from a file.
    pub fn load(&mut self) -> Result<StateDoc, String> {
        if self.path.is_dir() {
            return Err(String::from("State file is a directory"));
        }
        let file = self.open_file()?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| format!("Could not read state file: {}", e))?;
        match serde_json::from_str(&content) {
            Ok(states) => Ok(states),
            Err(e) => Err(format!("Could not parse state file: {}", e)),
        }
    }

    /// Save the state to a state file.
    /// # Arguments
    /// * `state` - State to be saved
    pub fn save(&mut self, state: &StateDoc) -> Result<(), String> {
        let content = serde_json::to_string_pretty(state)
            .map_err(|e| format!("Could not encode state file: {}", e))?;
        let file = self.open_file()?;
        file.seek(SeekFrom::Start(0))
            .map_err(|_| String::from("Could not jump to state file start."))?;
        file.set_len(0)
            .map_err(|_| String::from("Could not truncate state file."))?;
        match file.write_all(content.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Could not write to state file: {}", e)),
        }
    }

    /// Open or get the state file handle.
    fn open_file(&mut self) -> Result<&mut File, String> {
        match self.file.as_ref() {
            None => {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(self.path.as_path())
                    .map_err(|e| format!("Could not open state file: {}", e))?;
                file.lock_exclusive()
                    .map_err(|e| format!("Could not lock state file: {}", e))?;
                self.file = Some(file);
            }
            _ => (),
        };
        Ok(self.file.as_mut().unwrap())
    }

    /// Close state file handle.
    pub fn close_file(&mut self) -> Result<(), String> {
        let file = self.open_file()?;
        match file.unlock() {
            Ok(()) => {
                self.file = None;
                Ok(())
            }
            Err(_) => Err(String::from("Could not unlock state file.")),
        }
    }
}
