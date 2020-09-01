use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub path: std::path::PathBuf,
    pub size: u64,
    pub line_number: usize,
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
            line_number: 0,
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
            Ok(states_content) => match toml::from_str(&states_content) {
                Ok(states) => Ok(states),
                Err(e) => Err(format!("Could not parse statefile: {}", e))
            },
            _ => Err(String::from("Could not read statefile"))
        }
    }
    pub fn save(&self, state: &StateDoc) -> Result<(), String> {
        let content = match toml::to_string(state) {
            Ok(content) => content,
            Err(err) => return Err(format!("Could not encode statefile: {}", err))
        };
        match std::fs::write(&self.file, content) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!("Could not save statefile: {}", err))
        }
    }
}