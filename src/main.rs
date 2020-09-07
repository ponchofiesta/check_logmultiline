/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

//! Check_logmultiline is a log file check application to be used by Nagios or Icinga.

#[macro_use]
extern crate clap;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate directories;

mod args;
mod logfile;
mod state;

/// The name of this check printed for result output.
static RESULT_NAME: &str = "LOGFILES";

/// Exits program with UNKNOWN state.
/// # Arguments
/// * `msg` - The status message to be printed out
fn unknown(msg: &str) -> ! {
    println!("{} {:?}: {}", RESULT_NAME, crate::logfile::PatternType::UNKNOWN, msg);
    std::process::exit(crate::logfile::PatternType::UNKNOWN as i32);
}

fn main() {

    // Parse and validate command line arguments
    let args = match args::Args::parse() {
        Ok(args) => args,
        Err(e) => unknown(&format!("Could not parse command line arguments: {}", e))
    };

    // Get state of log file searches
    let state_loader = state::StateLoader::new(args.state_path);
    let mut statedoc = match state_loader.load() {
        Ok(states) => states,
        Err(e) => unknown(&format!("Could not load state: {}", e))
    };

    let mut results: Vec<logfile::Matches> = vec![];

    // Iterate through log files
    for file in args.files {

        // Get the state of the current log file
        let mut state = match statedoc.states.iter_mut().find(|state| state.path == file[0]) {
            Some(state) => state,
            None => {
                let state = state::State::new(file[0].clone());
                statedoc.states.push(state);
                statedoc.states.last_mut().unwrap()
            }
        };

        // Search the log file for defined patterns
        let result = match logfile::find(&file, state, &args.line_re, &args.patterns) {
            Ok(result) => result,
            Err(e) => unknown(&format!("Could not check log file: {}", e))
        };

        // Save results
        state.line_number = result.last_line_number;
        state.size = result.file_size;
        state.created = std::fs::metadata(&file[0]).unwrap().created().unwrap();

        results.push(result);
    }

    // Save new log file state
    if let Err(e) = state_loader.save(&statedoc) {
        unknown(&format!("Could not save state file: {}", e));
    };
 
    // Check results
    let mut code = logfile::PatternType::OK;
    if results.iter().any(|result| result.messages.iter().any(|message| message.message_type == crate::logfile::PatternType::CRITICAL)) {
        code = logfile::PatternType::CRITICAL;
    } else if results.iter().any(|result| result.messages.iter().any(|message| message.message_type == crate::logfile::PatternType::WARNING)) {
        code = logfile::PatternType::WARNING;
    }

    // Generate output message for results
    let mut msg = String::from(RESULT_NAME);
    msg.push_str(&format!(" {}: ", code));

    msg.push_str(&format!("{} warnings and {} criticals in {} lines of {} files\n", 
        results.iter().fold(0, 
            |count, matches| count + matches.messages.iter().filter(
                |message| message.message_type == crate::logfile::PatternType::WARNING
            ).count()
        ),
        results.iter().fold(0, 
            |count, matches| count + matches.messages.iter().filter(
                |message| message.message_type == crate::logfile::PatternType::CRITICAL
            ).count()
        ),
        results.iter().fold(0, |count, matches| count + matches.lines_count),
        results.iter().len()));

    for matches in results.iter() {
        if matches.messages.len() > 0 {
            msg.push_str(&matches.to_string());
        }
    }

    // Print output message and exit
    print!("{}", msg);
    std::process::exit(code as i32);
}
