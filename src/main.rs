/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Check_logmultiline is a log file check application to be used by Nagios or Icinga.

#[macro_use]
extern crate clap;
extern crate chrono;
extern crate directories;
extern crate fs2;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod args;
mod logfile;
mod state;

use args::Args;
use chrono::{prelude::*, Duration};
use logfile::{find, Match, ProblemType};
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
    let mut state_loader = StateLoader::new(args.state_path.as_path());
    let mut statedoc = match state_loader.load() {
        Ok(states) => states,
        Err(e) => unknown(&format!("Could not load state: {}", e)),
    };

    let mut matches: Vec<Match> = vec![];

    // Iterate through log files
    for file in &args.files {
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
        let mut matchh = match find(&file, state, &args.line_re, &args.patterns) {
            Ok(result) => result,
            Err(e) => unknown(&format!("Could not check log file: {}", e)),
        };

        // Clean up expired kept messages
        let now = Utc::now();
        state.kept_matches.retain(|matchh| matchh.keep_until >= now);

        // Keep messages in state
        if args.keep_status > 0 && matchh.messages.iter().len() > 0 {
            matchh.keep_until = now + Duration::seconds(args.keep_status);
            state.kept_matches.push(matchh.clone());
        }

        // Fill up state
        state.line_number = matchh.last_line_number;
        state.size = matchh.file_size;
        state.created = match metadata(&file[0]) {
            Ok(metadata) => match metadata.created() {
                Ok(created) => created,
                Err(e) => unknown(&format!(
                    "Could not get metadata of file {:?}: {}",
                    &file[0], e
                )),
            },
            Err(e) => unknown(&format!(
                "Could not get metadata of file {:?}: {}",
                &file[0], e
            )),
        };

        matches.push(matchh);
    }

    // Save log file state
    if let Err(e) = state_loader.save(&statedoc) {
        unknown(&format!("Could not save state file: {}", e));
    };
    if let Err(e) = state_loader.close_file() {
        unknown(&format!("Could not close state file: {}", e));
    }

    // Check kept messages
    let kept_matches: Vec<&Match> = statedoc
        .states
        .iter()
        .filter(|state| args.files.iter().any(|file| state.path == file[0]))
        .map(|state| &state.kept_matches)
        .flatten()
        .collect();
    let is_kept_critical = kept_matches.iter().any(|matches| matches.any_critical());
    let is_kept_warning = kept_matches.iter().any(|matches| matches.any_warning());

    // Check current results and set status code
    let is_critical = matches.iter().any(|matchh| matchh.any_critical());
    let is_warning = matches.iter().any(|matchh| matchh.any_warning());

    let code = if is_critical || is_kept_critical {
        ProblemType::CRITICAL
    } else if is_warning || is_kept_warning {
        ProblemType::WARNING
    } else {
        ProblemType::OK
    };

    // Generate output message for results
    let mut msg = String::from(RESULT_NAME);
    msg.push_str(&format!(" {}: ", code));

    // Get summary infomations
    let kept_warnings_count = kept_matches
        .iter()
        .fold(0, |count, matchh| count + matchh.count_warning());
    let kept_criticals_count = kept_matches
        .iter()
        .fold(0, |count, matchh| count + matchh.count_critical());

    let warnings_count = matches
        .iter()
        .fold(0, |count, matchh| count + matchh.count_warning());
    let criticals_count = matches
        .iter()
        .fold(0, |count, matchh| count + matchh.count_critical());
    let lines_count = matches
        .iter()
        .fold(0, |count, matchh| count + matchh.lines_count);
    let files_count = matches.iter().len();

    msg.push_str(&format!(
        "{} criticals and {} warnings - new: {} criticals and {} warnings in {} lines of {} files\n",
        kept_criticals_count, kept_warnings_count, criticals_count, warnings_count, lines_count, files_count
    ));

    // Print messages
    // Kept messages contains new messages here too
    if args.keep_status > 0 {
        for matches in kept_matches.iter() {
            if matches.messages.len() > 0 {
                msg.push_str(&matches.to_string());
            }
        }
    } else {
        for matches in matches.iter() {
            if matches.messages.len() > 0 {
                msg.push_str(&matches.to_string());
            }
        }
    }

    // Performance data
    msg.push_str(&format!(
        "|critical={} warning={} lines={}",
        criticals_count, warnings_count, lines_count
    ));

    // Print output message and exit
    println!("{}", msg.trim());
    exit(code as i32);
}
