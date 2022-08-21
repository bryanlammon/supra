#[macro_use]
extern crate slog;

mod userjournals;

use ansi_term::Color;
use clap::{crate_version, App, Arg, SubCommand};
use slog::{debug, Drain, Level};
use std::{fs, fs::OpenOptions, path::Path, process, sync::Mutex};

fn main() {
    // Get the command-line arguments and flags
    let matches = App::new("supra")
        .version(crate_version!())
        .author("Bryan Lammon")
        .about("A Pandoc preprocessor for legal scholarship")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("INPUT FILE")
                .help("The Pandoc markdown file to process")
                .index(1),
        )
        .arg(
            Arg::with_name("library")
                .short("l")
                .long("library")
                .value_name("LIBRARY FILE")
                .help("The reference library file in CSL JSON format")
                .index(2),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("OUTPUT FILE")
                .help("Optional output file (blank outputs to terminal)")
                .index(3),
        )
        .arg(
            Arg::with_name("offset")
                .short("f")
                .long("offset")
                .value_name("NUMBER")
                .help("The offset for counting footntes")
                .default_value("0")
                .allow_hyphen_values(true)
                .validator(is_number),
        )
        .arg(
            Arg::with_name("user journals")
                .short("u")
                .long("user_journals")
                .value_name("JOURNAL FILE")
                .help("A RON file containing user-provided short-journal names"),
        )
        .arg(
            Arg::with_name("force overwrite")
                .short("W")
                .long("force_overwrite")
                .takes_value(false)
                .help("Required to overwrite the input file with the output"),
        )
        .arg(
            Arg::with_name("smallcaps")
                .short("s")
                .long("smallcaps")
                .takes_value(false)
                .help("Applies a Word style called \"True small caps\" to all bolded text"),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .takes_value(false)
                .help("Outputs debug log to supra-log.json")
                .hidden_short_help(true)
                .hidden_long_help(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .value_name("NUMBER")
                .help("Verbosity level between 0 and 3")
                .hidden_short_help(true)
                .hidden_long_help(true)
                .default_value("2"),
        )
        .subcommand(SubCommand::with_name("uj").about("For creating a blank user-journals file"))
        .get_matches();

    // Setup the terminal logger.
    //
    // First determine the log level for debug builds. Then setup the terminal
    // logger. Finally, if the debug flag is set, setup the file longer
    // alongside the terminal logger.
    let min_log_level = match matches.value_of("verbose").unwrap() {
        "0" => Level::Critical,
        "1" => Level::Error,
        "2" => Level::Warning,
        "3" => Level::Info,
        "4" => Level::Debug,
        "5" => Level::Trace,
        _ => Level::Info,
    };
    let debug = matches.is_present("debug");

    let term_decorator = slog_term::TermDecorator::new().build();
    let term_drain = slog_term::CompactFormat::new(term_decorator).build().fuse();
    let term_drain = term_drain.filter_level(min_log_level).fuse();

    let _guard: slog_scope::GlobalLoggerGuard = if debug {
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("./supra-log.json")
            .unwrap();
        let file_drain = slog_json::Json::new(log_file)
            .set_pretty(true)
            .add_default_keys()
            .build()
            .fuse();
        let file_drain = file_drain.filter_level(Level::Trace).fuse();
        let dual_logger = slog::Logger::root(
            Mutex::new(slog::Duplicate(term_drain, file_drain)).fuse(),
            o!("version" => crate_version!()),
        );
        slog_scope::set_global_logger(dual_logger)
    } else {
        let term_logger = slog::Logger::root(
            Mutex::new(term_drain).fuse(),
            o!("version" => crate_version!()),
        );
        slog_scope::set_global_logger(term_logger)
    };

    debug!(slog_scope::logger(), "Logger setup");

    // Subcommands
    if let Some("uj") = matches.subcommand_name() {
        debug!(slog_scope::logger(), "Creating blank user-journal file");
        userjournals::new_user_journals_ron();
        process::exit(0);
    }

    // Setup the configuration variables.
    let input_file = Path::new(matches.value_of("input").unwrap());
    let library_file = Path::new(matches.value_of("library").unwrap());
    let output_option = matches.value_of("output").map(Path::new);
    let offset = matches.value_of("offset").unwrap().parse::<i32>().unwrap();
    let user_journals_file = if matches.is_present("user journals") {
        Some(Path::new(matches.value_of("user journals").unwrap()))
    } else {
        None
    };
    let smallcaps = matches.is_present("smallcaps");
    let force_overwrite = matches.is_present("force overwrite");

    // Deal with command-line errors.
    //
    // If the input and output strings are identical and force_overwrite has not
    // been used, return an error and exit.
    if output_option.is_some() && input_file == output_option.unwrap() && !force_overwrite {
        eprintln!("The input file\n\t({})\n and output file\n\t({})\nare identical, but the force overwrite flag (-W/--force_overwrite) was not set.\nIf you want to overewrite the input file, use -W/--force_overwrite", Color::Blue.paint(input_file.to_string_lossy()), Color::Blue.paint(output_option.unwrap().to_string_lossy()));
        process::exit(1);
    }

    // Load the inputs.
    let input =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "load_file()")), || {
            load_file(input_file)
        }) {
            Ok(i) => i,
            Err(e) => {
                error!(slog_scope::logger(), "Markdown load error: {}", e);
                eprintln!("Markdown load error: {}", e);
                process::exit(1);
            }
        };

    // Load the CSL JSON library
    let library =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "load_file()")), || {
            load_file(library_file)
        }) {
            Ok(l) => l,
            Err(e) => {
                error!(slog_scope::logger(), "CSL JSON load error: {}", e);
                eprintln!("CSL JSON load error: {}", e);
                process::exit(1);
            }
        };

    let user_journals = match user_journals_file {
        Some(u) => {
            match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "load_file()")), || {
                load_file(u)
            }) {
                Ok(j) => Some(j),
                Err(e) => {
                    error!(slog_scope::logger(), "User journals load error: {}", e);
                    eprintln!("User journals load error: {}", e);
                    process::exit(1);
                }
            }
        }
        None => None,
    };

    // Set the options
    let options = supra::SupraOptions::new(offset, user_journals, smallcaps);

    // Run the program.
    let output = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "supra()")), || {
        supra::supra(&input, &library, options)
    }) {
        Ok(o) => o,
        Err(e) => {
            error!(slog_scope::logger(), "Application error: {}", e);
            drop(_guard);
            eprintln!("Application error: {}", e);
            process::exit(1);
        }
    };

    // Output the result
    slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "output_result()")),
        || output_result(output_option, &output),
    );
}

/// Ensure that the command-line value is an integer.
fn is_number(f: String) -> Result<(), String> {
    if f.parse::<i32>().is_ok() {
        return Ok(());
    }
    Err(String::from("The offset must be an integer"))
}

/// Load the a file into a string.
///
/// This function is used to load both the markdown and CSL JSON files into
/// strings, which can then be passed to the main function.
fn load_file(path: &Path) -> Result<String, String> {
    debug!(
        slog_scope::logger(),
        "Loading file {}...",
        path.to_string_lossy()
    );

    match fs::read_to_string(path) {
        Ok(r) => {
            debug!(
                slog_scope::logger(),
                "File {} loaded.",
                path.to_string_lossy()
            );
            Ok(r)
        }
        Err(e) => {
            let err_msg = format!("error reading the file {}—{}", path.to_string_lossy(), e);
            Err(err_msg)
        }
    }
}

/// Output the result to a file or the terminal.
fn output_result(output_option: Option<&Path>, output: &str) {
    debug!(slog_scope::logger(), "Outputting results...");

    match output_option {
        Some(f) => {
            fs::write(f, output).expect("Unable to write file");
        }
        None => {
            println!("{}", output);
        }
    }

    debug!(slog_scope::logger(), "Results output.");
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_validators {
        use super::*;

        #[test]
        fn numerical_offset() {
            let offset = is_number("-1".to_string());
            assert!(offset.is_ok());
        }

        #[test]
        fn non_numerical_offset() {
            let offset = is_number("banana".to_string());
            assert!(offset.is_err());
        }
    }

    mod test_load_file {
        use super::*;

        #[test]
        fn test_load() {
            let file = "./tests/test-input.md";
            let load_result = load_file(Path::new(file));
            assert!(load_result.is_ok());
            assert!(load_result.unwrap().contains("Supra Test Document"));
        }

        #[test]
        fn fail_load() {
            let file = "./tests/does-not-exist.md";
            let load_result = load_file(Path::new(file));
            assert!(load_result
                .unwrap_err()
                .contains("No such file or directory"));
        }
    }
}
