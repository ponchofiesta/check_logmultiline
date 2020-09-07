/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Analyze log files.

use std::io::BufRead;

/// A tuple containing the type of the pattern and the pattern.
pub type Pattern = (PatternType, regex::Regex);

/// The struct contains the informations about matches in a log file.
pub struct Matches {

    /// Path to the log file.
    pub path: std::path::PathBuf,

    /// The count of lines that has been analyzed.
    pub lines_count: usize,

    /// Last line number that has been analyzed.
    pub last_line_number: i64,

    /// Size of the log file.
    pub file_size: u64,

    /// Matching messages.
    pub messages: Vec<Message>,
}

/// A multiline message from a log file.
#[derive(Clone)]
pub struct Message {

    /// The line number the message started in.
    pub line_number: i64,

    /// Type of pattern found.
    pub message_type: PatternType,

    /// The message string.
    pub message: String,
}

/// The type of pattern or problem.
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

    /// Create a new default Message.
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

/// Search the log file set for specific patterns and return the matches.
/// # Arguments
/// * `files` - A file set of log files to be searched through
/// * `state` - The state of the log file
/// * `line_re` - The line pattern to determine message starts
/// * `patterns` - Patterns to search for in the log files
pub fn find(
    files: &crate::args::Files, 
    state: &crate::state::State, 
    line_re: &regex::Regex, 
    patterns: &Vec<Pattern>) -> Result<Matches, String> {

    // Find last used log file
    let mut file_selector = files.iter().len();
    for (index, file) in files.iter().enumerate() {
        let created = std::fs::metadata(file).unwrap().created().unwrap();
        let size = std::fs::metadata(file).unwrap().len();
        if state.created == created && state.size <= size {
            file_selector = index;
            break;
        }
    }

    let mut matches = Matches {
        path: state.path.clone(),
        lines_count: 0,
        last_line_number: state.line_number,
        file_size: std::fs::metadata(&files[0]).unwrap().len(),
        messages: vec![]
    };

    // Walk through all log files to current
    for index in (0..=file_selector).rev() {
        let file = match std::fs::File::open(&files[index]) {
            Ok(file) => file,
            Err(e) => return Err(format!("Could not search in log file: {}", e))
        };
        let reader = std::io::BufReader::new(file);
        let mut message = Message::new();
        
        for (index, line) in reader.lines().enumerate() {
    
            let index = index as i64;

            // Skip to first unseen line
            if index <= state.line_number {
                continue;
            }
    
            message.line_number = index;
            let line = line.unwrap();
            
            if line_re.is_match(&line) {
    
                // last message has finished, analyze it
                find_in_message(&mut message, patterns, &mut matches);
                
                // new message starts
                message = Message::new();
            }
            
            message.message.push_str(&format!("{}\n", line));
            matches.lines_count += 1;
            matches.last_line_number = index;
        }
        find_in_message(&mut message, patterns, &mut matches);
    }
    
    Ok(matches)
}

/// Search patterns in single message.
/// # Arguments
/// * `message` - The message to search through
/// * `patterns` - Patterns to search for in the message
/// * `line_re` - Store matching messages in this struct
fn find_in_message(message: &mut Message, patterns: &Vec<Pattern>, matches: &mut Matches) {
    for re in patterns {
        if re.1.is_match(&message.message) {
            message.message_type = re.0;
            matches.messages.push(message.clone());
        }
    }
}