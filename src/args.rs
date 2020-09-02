/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

 pub struct Args {
    pub files: Vec<std::path::PathBuf>,
    pub line_re: regex::Regex,
    pub patterns: Vec<crate::logfile::Pattern>,
    pub state_path: std::path::PathBuf,
}

impl Args {
    pub fn parse() -> Result<Self, String> {

        let args = clap_app!(app => 
            (name: crate_name!())
            (version: crate_version!())
            (author: crate_authors!())
            (about: "Checks log files for specific patterns and respects messages with multiple lines")
            (@arg file: -f --file +takes_value +required +multiple "Log file to analyze")
            (@arg linepattern: -l --line +takes_value "Pattern to detect new lines")
            (@arg warningpattern: -w --warningpattern +takes_value +multiple "Regex pattern to trigger a WARNING problem")
            (@arg criticalpattern: -c --criticalpattern +takes_value +multiple "Regex pattern to trigger a CRITICAL problem")
            (@arg statefile: -s --statefile +takes_value "File to save the processing state in from run to run")
        ).get_matches();

        // file
        let files: Vec<_> = args.values_of("file").unwrap().collect();

        // linepattern
        let linepattern = args.value_of("linepattern").unwrap_or("");
        let line_re = match regex::Regex::new(linepattern) {
            Ok(re) => re,
            Err(e) => return Err(format!("Invalid line pattern: {}", e)),
        };

        // warningpattern
        let mut patterns: Vec<crate::logfile::Pattern> = vec![];

        let warningpatterns: Vec<_> = match args.values_of("warningpattern") {
            Some(values) => values.collect(),
            None => vec![],
        };
        for pattern in warningpatterns {
            match regex::Regex::new(pattern) {
                Ok(re) => patterns.push((crate::logfile::PatternType::WARNING, re)),
                Err(e) => return Err(format!("Invalid warning pattern: {}", e)),
            };
        }

        // criticalpattern
        let criticalpatterns: Vec<_> = match args.values_of("criticalpattern") {
            Some(values) => values.collect(),
            None => vec![],
        };
        for pattern in criticalpatterns {
            match regex::Regex::new(pattern) {
                Ok(re) => patterns.push((crate::logfile::PatternType::CRITICAL, re)),
                Err(e) => return Err(format!("Invalid critical pattern: {}", e)),
            };
        }

        // statefile
        let statepath = match args.value_of("statefile") {
            Some(value) => std::path::PathBuf::from(value),
            None => match crate::directories::ProjectDirs::from("de", "osor", crate_name!()) {
                Some(proj) => proj.data_dir().to_path_buf(),
                None => {
                    let mut statepath = std::env::temp_dir();
                    statepath.push(format!("{}_state.toml", crate_name!()));
                    statepath
                },
            },
        };

        Ok(Args{
            files: files.iter().map(|&file| std::path::PathBuf::from(file)).collect(),
            line_re: line_re,
            patterns: patterns,
            state_path: std::path::PathBuf::from(statepath),
        })
    }
}