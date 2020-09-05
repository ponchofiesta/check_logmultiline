# check_logmultiline

A Check for Nagios or Icinga to analyze log files.

Check_logmultiline searches log files message by message. It saves its state in a state file to scan only new lines in a log file.

## Features

- Multiline log messages (detected by user defined regex pattern)
- Multiple log files
- Rotating log files
- Multiple warning and critical patterns

## Prerequisites

- glibc >= 2.18 (Linux; or build on your own to support other versions)

## Run

Help:

```
USAGE:
    check_logmultiline [OPTIONS] --file <file>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --criticalpattern <criticalpattern>...    Regex pattern to trigger a CRITICAL problem
    -f, --file <file>...
            Log file to analyze. Append ':<filenamepattern>' to specify rotated files.

    -l, --line <linepattern>                      Pattern to detect new lines
    -s, --statefile <statefile>                   File to save the processing state in from run to run
    -w, --warningpattern <warningpattern>...      Regex pattern to trigger a WARNING problem
```

### Examples

Check for a specific pattern of Java stacktraces in log files:

```bash
check_logmultiline -f /var/log/someapp.log -l '^\[.*?\] [\da-f]{8} ' -c 'java\.lang\.OutOfMemoryError'
```

Check every line in rotating log files:

```bash
check_logmultiline -f '/var/log/someapp.log:^someapp\.\d\.log' -c 'java\.lang\.OutOfMemoryError'
```

## Build

You only need Rust edition 2018 (version >= 1.31). And run:

```bash
cargo build --release
```

## License

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.