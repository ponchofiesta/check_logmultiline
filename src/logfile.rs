/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Analyze log files.

use crate::args::Files;
use crate::state::State;
use chrono::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::fs::{metadata, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::SystemTime;

/// A tuple containing the type of the pattern and the pattern.
pub type Pattern = (ProblemType, Regex);

/// The struct contains the informations about matches in a log file.
#[derive(Clone, Serialize, Deserialize)]
pub struct Match {
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

    /// The date til when the message should be kept if keep_status is active.
    pub keep_until: DateTime<Utc>,
}

/// A multiline message from a log file.
#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    /// The line number the message started in.
    pub line_number: i64,

    /// Type of pattern found.
    pub message_type: ProblemType,

    /// The message string.
    pub message: String,
}

/// The type of pattern or problem.
#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub enum ProblemType {
    OK = 0,
    WARNING = 1,
    CRITICAL = 2,
    UNKNOWN = 3,
}

impl Display for Match {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        output.push_str(&format!("File: {}\n", self.path.to_str().unwrap()));

        for message in &self.messages {
            output.push_str(&format!("{}\n", message));
        }

        write!(f, "{}", output)
    }
}

impl Match {
    /// Tests if any message is CRITICAL.
    pub fn any_critical(&self) -> bool {
        self.messages
            .iter()
            .any(|message| message.message_type == ProblemType::CRITICAL)
    }

    /// Tests if any message is WARNING.
    pub fn any_warning(&self) -> bool {
        self.messages
            .iter()
            .any(|message| message.message_type == ProblemType::WARNING)
    }

    /// Counts the CRITICAL messages.
    pub fn count_critical(&self) -> usize {
        self.messages
            .iter()
            .filter(|message| message.message_type == ProblemType::CRITICAL)
            .count()
    }

    /// Counts the WARNING messages.
    pub fn count_warning(&self) -> usize {
        self.messages
            .iter()
            .filter(|message| message.message_type == ProblemType::WARNING)
            .count()
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}): {}",
            self.message_type, self.line_number, self.message
        )
    }
}

impl Message {
    /// Create a new default Message.
    pub fn new() -> Self {
        Message {
            line_number: 0,
            message_type: ProblemType::UNKNOWN,
            message: String::new(),
        }
    }
}

impl Display for ProblemType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    files: &Files,
    state: &State,
    line_re: &Regex,
    patterns: &Vec<Pattern>,
) -> Result<Match, String> {
    // Find last used log file
    let mut file_selector = files.iter().len() - 1;
    for (index, file) in files.iter().enumerate() {
        let file_time = file_modified(file.as_path())?;
        if state.modified >= file_time {
            break;
        }
        file_selector = index;
    }

    let mut matches = Match {
        path: state.path.clone(),
        lines_count: 0,
        last_line_number: state.line_number,
        file_size: metadata(&files[0]).unwrap().len(),
        messages: vec![],
        keep_until: Utc::now(),
    };

    // Walk through all log files to current
    for file_index in (0..=file_selector).rev() {
        let file = File::open(&files[file_index])
            .map_err(|e| format!("Could not search in log file: {}", e))?;
        let reader = BufReader::new(file);
        let mut message = Message::new();
        let mut iterator = reader.lines().enumerate();
        while let Some((line_index, Ok(line))) = iterator.next() {

            let line_index = line_index as i64;

            // Skip to first unseen line
            if line_index <= state.line_number {
                continue;
            }
            message.line_number = line_index;
            if line_re.is_match(&line) {
                // last message has finished, analyze it
                find_in_message(&mut message, patterns, &mut matches);
                // new message starts
                message = Message::new();
            }
            message.message.push_str(&format!("{}\n", line));
            matches.lines_count += 1;
            matches.last_line_number = line_index;
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
fn find_in_message(message: &mut Message, patterns: &Vec<Pattern>, matches: &mut Match) {
    for re in patterns {
        if re.1.is_match(&message.message) {
            message.message_type = re.0;
            matches.messages.push(message.clone());
        }
    }
}

/// Get file modified time.
/// # Arguments
/// * `path` - The file path to get time from
pub fn file_modified(path: &Path) -> Result<SystemTime, String> {
    let file_time = metadata(path)
        .map_err(|e| format!("Could not get file metadata: {}", e))?
        .modified()
        .map_err(|e| format!("Could not get modified date: {}", e))?;
    Ok(file_time)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        // given
        let mut message = Message {
            line_number: 1,
            message_type: ProblemType::OK,
            message: "abc 123".into(),
        };
        let patterns = vec![(ProblemType::CRITICAL, Regex::new(r"123").unwrap())];
        let mut matches = Match {
            path: std::path::PathBuf::new(),
            lines_count: 0,
            last_line_number: 1,
            file_size: 123,
            messages: vec![],
            keep_until: Utc::now(),
        };
        // when
        find_in_message(&mut message, &patterns, &mut matches);

        // then
        assert_eq!(message.message_type, ProblemType::CRITICAL);
        assert_eq!(matches.messages.len(), 1);
    }
}
