/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Check_logmultiline is a log file check application to be used by Nagios or Icinga.

#[macro_use]
extern crate clap;
extern crate directories;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod args;
mod logfile;
mod state;

use args::Args;
use logfile::{find, Matches, ProblemType};
use state::{State, StateLoader};
use std::fs::metadata;
use std::process::exit;

/// The name of this check printed for result output.
static RESULT_NAME: &str = "LOGFILES";

/// Exits program with UNKNOWN state.
/// # Arguments
/// * `msg` - The status message to be printed out
fn unknown(msg: &str) -> ! {
    println!("{} {:?}: {}", RESULT_NAME, ProblemType::UNKNOWN, msg);
    exit(ProblemType::UNKNOWN as i32);
}

fn main() {
    // Parse and validate command line arguments
    let args = match Args::get() {
        Ok(args) => args,
        Err(e) => unknown(&format!("Could not parse command line arguments: {}", e)),
    };

    // Get state of log file searches
    let state_loader = StateLoader::new(args.state_path);
    let mut statedoc = match state_loader.load() {
        Ok(states) => states,
        Err(e) => unknown(&format!("Could not load state: {}", e)),
    };

    let mut results: Vec<Matches> = vec![];

    // Iterate through log files
    for file in args.files {
        // Get the state of the current log file
        let mut state = match statedoc
            .states
            .iter_mut()
            .find(|state| state.path == file[0])
        {
            Some(state) => state,
            None => {
                let state = State::new(file[0].clone());
                statedoc.states.push(state);
                statedoc.states.last_mut().unwrap()
            }
        };

        // Search the log file for defined patterns
        let result = match find(&file, state, &args.line_re, &args.patterns) {
            Ok(result) => result,
            Err(e) => unknown(&format!("Could not check log file: {}", e)),
        };

        // Save results
        state.line_number = result.last_line_number;
        state.size = result.file_size;
        state.created = metadata(&file[0]).unwrap().created().unwrap();

        results.push(result);
    }

    // Save new log file state
    if let Err(e) = state_loader.save(&statedoc) {
        unknown(&format!("Could not save state file: {}", e));
    };

    // Check results and set status code
    let is_critical = results.iter().any(|result| result.any_critical());
    let is_warning = results.iter().any(|result| result.any_warning());

    let code = if is_critical {
        ProblemType::CRITICAL
    } else if is_warning {
        ProblemType::WARNING
    } else {
        ProblemType::OK
    };

    // Generate output message for results
    let mut msg = String::from(RESULT_NAME);
    msg.push_str(&format!(" {}: ", code));

    let warnings_count = results
        .iter()
        .fold(0, |count, matches| count + matches.count_warning());
    let criticals_count = results
        .iter()
        .fold(0, |count, matches| count + matches.count_critical());
    let lines_count = results
        .iter()
        .fold(0, |count, matches| count + matches.lines_count);
    let files_count = results.iter().len();

    msg.push_str(&format!(
        "{} warnings and {} criticals in {} lines of {} files\n",
        warnings_count, criticals_count, lines_count, files_count
    ));

    for matches in results.iter() {
        if matches.messages.len() > 0 {
            msg.push_str(&matches.to_string());
        }
    }

    // Print output message and exit
    print!("{}", msg);
    exit(code as i32);
}
