/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Parse and validate command line arguments.

use crate::logfile::{Pattern, ProblemType};
use directories::ProjectDirs;
use regex::Regex;
use std::env::temp_dir;
use std::fs::metadata;
use std::fs::read_dir;
use std::path::PathBuf;
use std::time::SystemTime;

/// Processed and transformed command line arguments.
pub struct Args {
    /// List of log file sets.
    pub files: Vec<Files>,

    /// Regular expression pattern to determine a message start.
    pub line_re: Regex,

    /// List of regular expressions to search for.
    pub patterns: Vec<Pattern>,

    /// The path to the state file.
    pub state_path: PathBuf,
}

/// A file set containing the main log file with index 0 and possible rotated log files following ordered by its creating date.
pub type Files = Vec<PathBuf>;

// A list of tuples containing the log file path and the corresponding file creation date.
type FilesCreated = Vec<(PathBuf, SystemTime)>;

impl Args {
    /// Parse, validate and transform the command line arguments.
    pub fn parse() -> Result<Self, String> {
        let args = clap_app!(app => (name: env!("CARGO_PKG_NAME"))
            (version: env!("CARGO_PKG_VERSION"))
            (author: env!("CARGO_PKG_AUTHORS"))
            (about: env!("CARGO_PKG_DESCRIPTION"))
            (@arg file: -f --file +takes_value +required +multiple "Log file to analyze. Append ':<filenamepattern>' to specify rotated files.")
            (@arg linepattern: -l --line +takes_value "Pattern to detect new lines")
            (@arg warningpattern: -w --warningpattern +takes_value +multiple "Regex pattern to trigger a WARNING problem")
            (@arg criticalpattern: -c --criticalpattern +takes_value +multiple "Regex pattern to trigger a CRITICAL problem")
            (@arg statefile: -s --statefile +takes_value "File to save the processing state in from run to run")
        ).get_matches();

        // file
        let files_arg: Vec<&str> = args.values_of("file").unwrap().collect();
        let mut all_files: Vec<Files> = vec![];
        for file_arg in files_arg {
            // Split file argument to get the path and a pattern for rotated file names
            let file_parts: Vec<&str> = file_arg.splitn(2, ':').collect();
            let path = PathBuf::from(file_parts[0]);
            let created = metadata(path.as_path()).unwrap().created().unwrap();
            let mut files: FilesCreated = vec![(path, created)];

            // Search for rotated log files
            if file_parts.iter().len() > 1 {
                let pattern = match Regex::new(file_parts[1]) {
                    Ok(pattern) => pattern,
                    Err(e) => return Err(format!("Invalid rotate log file pattern: {}", e)),
                };
                let parent_dir = match files[0].0.parent() {
                    Some(dir) => dir.to_path_buf(),
                    None => return Err(String::from("Log file path has no parent directory")),
                };
                if parent_dir.is_dir() {
                    for entry in read_dir(parent_dir.as_path()).unwrap() {
                        let filename = entry.unwrap().file_name().into_string().unwrap();
                        if pattern.is_match(&filename) {
                            let path = parent_dir.join(filename);
                            let created = metadata(path.as_path()).unwrap().created().unwrap();
                            files.push((path, created));
                        }
                    }
                }
            }

            // Sort file by "created" time to have the oldest last
            files.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            files.reverse();
            let files: Files = files.into_iter().map(|file| file.0).collect();

            all_files.push(files);
        }

        // linepattern
        let linepattern = args.value_of("linepattern").unwrap_or("");
        let line_re = match Regex::new(linepattern) {
            Ok(re) => re,
            Err(e) => return Err(format!("Invalid line pattern: {}", e)),
        };

        // warningpattern
        let mut patterns: Vec<Pattern> = vec![];

        let warningpatterns: Vec<_> = match args.values_of("warningpattern") {
            Some(values) => values.collect(),
            None => vec![],
        };
        for pattern in warningpatterns {
            match Regex::new(pattern) {
                Ok(re) => patterns.push((ProblemType::WARNING, re)),
                Err(e) => return Err(format!("Invalid warning pattern: {}", e)),
            };
        }

        // criticalpattern
        let criticalpatterns: Vec<_> = match args.values_of("criticalpattern") {
            Some(values) => values.collect(),
            None => vec![],
        };
        for pattern in criticalpatterns {
            match Regex::new(pattern) {
                Ok(re) => patterns.push((ProblemType::CRITICAL, re)),
                Err(e) => return Err(format!("Invalid critical pattern: {}", e)),
            };
        }

        // statefile
        let statepath = match args.value_of("statefile") {
            Some(value) => PathBuf::from(value),
            None => match ProjectDirs::from("de", "osor", crate_name!()) {
                Some(proj) => proj.data_dir().to_path_buf(),
                None => {
                    let mut statepath = temp_dir();
                    statepath.push(format!("{}_state.json", crate_name!()));
                    statepath
                }
            },
        };

        Ok(Args {
            files: all_files,
            line_re: line_re,
            patterns: patterns,
            state_path: PathBuf::from(statepath),
        })
    }
}
