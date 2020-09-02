/*
 * Copyright (c) 2020 Michael Richter <mr@osor.de>
 */

pub struct Args {
    pub files: Vec<Files>,
    pub line_re: regex::Regex,
    pub patterns: Vec<crate::logfile::Pattern>,
    pub state_path: std::path::PathBuf,
}

pub type Files = Vec<std::path::PathBuf>;

type FilesCreated = Vec<(std::path::PathBuf, std::time::SystemTime)>;

impl Args {
    pub fn parse() -> Result<Self, String> {

        let args = clap_app!(app => 
            (name: crate_name!())
            (version: crate_version!())
            (author: crate_authors!())
            (about: "Checks log files for specific patterns and respects messages with multiple lines")
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
            let file_parts: Vec<&str> = file_arg.splitn(2, ':').collect();
            let path = std::path::PathBuf::from(file_parts[0]);
            let created = std::fs::metadata(path.as_path()).unwrap().created().unwrap();
            let mut files: FilesCreated = vec![(path, created)];

            if file_parts.iter().len() > 1 {
                let pattern = match regex::Regex::new(file_parts[1]) {
                    Ok(pattern) => pattern,
                    Err(e) => return Err(format!("Invalid rotate log file pattern: {}", e)),
                };
                let parent_dir = match files[0].0.parent() {
                    Some(dir) => dir.to_path_buf(),
                    None => return Err(String::from("Log file path has no parent directory")),
                };
                if parent_dir.is_dir() {
                    for entry in std::fs::read_dir(parent_dir.as_path()).unwrap() {
                        let filename = entry.unwrap().file_name().into_string().unwrap();
                        if pattern.is_match(&filename) {
                            let path = parent_dir.join(filename);
                            let created = std::fs::metadata(path.as_path()).unwrap().created().unwrap();
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
                    statepath.push(format!("{}_state.json", crate_name!()));
                    statepath
                },
            },
        };

        Ok(Args{
            files: all_files,
            line_re: line_re,
            patterns: patterns,
            state_path: std::path::PathBuf::from(statepath),
        })
    }
}