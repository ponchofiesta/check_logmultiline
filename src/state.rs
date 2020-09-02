/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub path: std::path::PathBuf,
    pub size: u64,
    pub created: std::time::SystemTime,
    pub line_number: i64,
}

#[derive(Serialize, Deserialize)]
pub struct StateDoc {
    pub states: Vec<State>
}

impl StateDoc {
    pub fn new() -> Self {
        StateDoc {
            states: vec![]
        }
    }
}

pub struct StateLoader {
    file: std::path::PathBuf,
}

impl State {
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
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        StateLoader {
            file: path.as_ref().to_path_buf(),
        }
    }
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