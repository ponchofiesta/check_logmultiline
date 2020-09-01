use std::io::BufRead;

pub type Pattern = (PatternType, regex::Regex);

pub struct Matches {
    pub path: std::path::PathBuf,
    pub lines_count: usize,
    pub last_line_number: usize,
    pub file_size: u64,
    pub messages: Vec<Message>,
}

#[derive(Clone)]
pub struct Message {
    pub line_number: usize,
    pub message_type: PatternType,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PatternType {
    OK = 0,
    WARNING = 1,
    CRITICAL = 2,
    UNKNOWN = 3,
}

impl std::fmt::Display for Matches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        output.push_str(&format!("File: {}\n", self.path.to_str().unwrap()));

        for message in &self.messages {
            output.push_str(&format!("{}\n", message));
        }

        write!(f, "{}", output)
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}): {}", self.message_type, self.line_number, self.message)
    }
}

impl Message {
    pub fn new() -> Self {
        Message {
            line_number: 0,
            message_type: PatternType::UNKNOWN,
            message: String::new(),
        }
    }
}

impl std::fmt::Display for PatternType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn find(state: &crate::state::State, line_re: &regex::Regex, patterns: &Vec<Pattern>) -> Result<Matches, String> {

    let file = match std::fs::File::open(&state.path) {
        Ok(file) => file,
        Err(e) => return Err(format!("Could not search in log file: {}", e))
    };
    let reader = std::io::BufReader::new(file);

    let mut message = Message::new();
    let mut matches = Matches {
        path: state.path.clone(),
        lines_count: 0,
        last_line_number: state.line_number,
        file_size: std::fs::metadata(&state.path).unwrap().len(),
        messages: vec![]
    };

    for (index, line) in reader.lines().enumerate() {

        if index < state.line_number {
            continue;
        }

        let line = line.unwrap();
        
        if line_re.is_match(&line) {

            // last message has finished, analyze it
            for re in patterns {
                if re.1.is_match(&message.message) {
                    message.message_type = re.0;
                    matches.messages.push(message.clone());
                }
            }
            
            // new message starts
            message = Message::new();
            message.line_number = index;
        }
        
        message.message.push_str(&format!("{}\n", line));
        matches.lines_count += 1;
        matches.last_line_number = index;
    }

    Ok(matches)
}