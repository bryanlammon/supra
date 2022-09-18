//! `main.rs` contains the command-line interface for supra. It collects the
//! values and options, sets up the logger, assembles the configuration, and
//! passes everything to the main function.

#[macro_use]
extern crate slog;

use ansi_term::Color;
use clap::{crate_version, value_parser, App, Arg, SubCommand};
use slog::{debug, Drain, Level};
use std::{fs::OpenOptions, process, sync::Mutex};
use supra::config::{Output, PanConfig, PostConfig, PreConfig, SupraCommand, SupraConfig};

fn main() -> Result<(), String> {
    // Get the command-line arguments and options
    let matches = App::new("supra")
        .version(crate_version!())
        .author("Bryan Lammon")
        .about("A Pandoc wrapper for legal scholarship")
        .subcommand_negates_reqs(true)
        .arg(
            Arg::with_name("input")
                .value_name("INPUT FILE")
                .help("The Pandoc markdown file to process")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("library")
                .value_name("LIBRARY FILE")
                .help("The reference library file in CSL JSON format")
                .index(2)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .value_name("OUTPUT FILE")
                .help("The .md or .docx output (blank oputputs to terminal)")
                .index(3)
                .required(false),
        )
        .arg(
            Arg::with_name("pandoc_reference")
                .value_name("REFERENCE FILE")
                .help("The Pandoc custom reference file")
                .index(4)
                .required(false),
        )
        .arg(
            Arg::with_name("smallcaps")
                .short('s')
                .long("smallcaps")
                .takes_value(false)
                .help("Applies a Word style called \"Small caps\" to all bolded text"),
        )
        .arg(
            Arg::with_name("offset")
                .short('o')
                .long("offset")
                .value_name("NUMBER")
                .help("The offset for counting footntes")
                .default_value("0")
                .allow_hyphen_values(true)
                .value_parser(value_parser!(usize)),
        )
        .arg(
            Arg::with_name("user_journals")
                .short('u')
                .long("user_journals")
                .value_name("JOURNAL FILE")
                .help("A RON file containing user-provided short-journal names"),
        )
        .arg(
            Arg::with_name("force_overwrite")
                .short('W')
                .long("force_overwrite")
                .takes_value(false)
                .help("Required to overwrite the input file with the output"),
        )
        .arg(
            Arg::with_name("autocref")
                .short('c')
                .long("autocref")
                .takes_value(false)
                .help("Turns cross-references in footnotes into Word fields"),
        )
        .arg(
            Arg::with_name("author_note")
                .short('a')
                .long("author")
                .help("Add an author footnote (requires .ron file with contents)"),
        )
        .arg(
            Arg::with_name("tabbed_footnotes")
                .short('t')
                .long("tabs")
                .help("Replace the space after footnote numbers with a tab"),
        )
        .arg(
            Arg::with_name("no_superscript")
                .short('n')
                .long("no_superscript")
                .help("Make footnotes non-superscript in the footnotes"),
        )
        .arg(
            Arg::with_name("running_header")
                .short('r')
                .long("header")
                .help(
                    "Add a year and running title to the header (requires .ron file with contents)",
                ),
        )
        .arg(
            Arg::with_name("debug")
                .short('d')
                .long("debug")
                .takes_value(false)
                .help("Outputs debug log to supra-log.json")
                .hidden_short_help(true)
                .hidden_long_help(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short('v')
                .long("verbose")
                .value_name("NUMBER")
                .help("Verbosity level between 0 and 3")
                .hidden_short_help(true)
                .hidden_long_help(true)
                .default_value("1"),
        )
        .subcommand(SubCommand::with_name("uj").about("Create a blank user-journals file"))
        .subcommand(
            SubCommand::with_name("new")
                .about("Create a new project")
                .arg(
                    // Although this takes only one value, allowing multiple
                    // values is useful for error handling. That way if the user
                    // provides a name with whitespaces, a custom error tells
                    // the user to not use whitespaces. (The clap error treats
                    // the words after the first word as arguments, which it
                    // wasn't expecting.)
                    Arg::with_name("name")
                        .value_name("NAME")
                        .help("The name of your new project")
                        .required(true)
                        .min_values(1),
                )
                .arg(
                    Arg::with_name("force_overwrite")
                        .short('W')
                        .long("force_overwrite")
                        .takes_value(false)
                        .help("Required to overwrite existing Markdown files and Makefiles"),
                ),
        )
        .get_matches();

    // Setup the logger.
    //
    // If the debug flag is set, the log is output to a file `supra-log.json`.
    // Otherwise, all logging goes to the terminal.
    let debug = matches.is_present("debug");
    let min_log_level = match matches.value_of("verbose").unwrap() {
        "0" => Level::Critical,
        "1" => Level::Error,
        "2" => Level::Warning,
        "3" => Level::Info,
        "4" => Level::Debug,
        "5" => Level::Trace,
        _ => Level::Info,
    };

    let term_decorator = slog_term::TermDecorator::new().build();
    let term_drain = slog_term::CompactFormat::new(term_decorator).build().fuse();
    let term_drain = term_drain.filter_level(min_log_level).fuse();

    let _guard: slog_scope::GlobalLoggerGuard = if debug {
        // Setup the file AND terminal loggers
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
        // Setup just the terminal logger
        let term_logger = slog::Logger::root(
            Mutex::new(term_drain).fuse(),
            o!("version" => crate_version!()),
        );
        slog_scope::set_global_logger(term_logger)
    };

    debug!(slog_scope::logger(), "Logger setup");

    // Determine which command to run
    let command = match matches.subcommand() {
        Some(("uj", _)) => SupraCommand::NewUserJournalFile,
        Some(("new", sub_matches)) => {
            // Get the name of the new project
            let vals: Vec<_> = sub_matches.values_of("name").unwrap().collect();

            // If there's more than one value, return an error. This is better
            // than clap's error, because it doesn't explain that the project
            // name must be a single word with no whitespaces.
            if vals.len() > 1 {
                eprintln!(
                    "{} Project names must be a single word with no whitespaces",
                    Color::Red.paint("ERRO")
                );
                process::exit(1);
            }

            // If there's only one value, use that
            let name = vals[0];

            // Error correction for invalid characters in the name.
            // # % & { } \ < > * ? / $ ! ' " : @ + ` | =
            if !valid_name(name) {
                eprintln!(
                    "{} Project name contains an invalid character\n     Do not use the following: {}",
                    Color::Red.paint("ERRO"),
                    Color::Red.paint("# % & { } \\ < > * ? / $ ! ' \" : @ + ` | =")
                );
                process::exit(1);
            }

            let overwrite = sub_matches.is_present("force_overwrite");

            SupraCommand::NewProject(name, overwrite)
        }
        _ => SupraCommand::Main,
    };

    let config = match command {
        SupraCommand::NewUserJournalFile => SupraConfig::new(command, None, None, None, None),
        SupraCommand::NewProject(_, _) => SupraConfig::new(command, None, None, None, None),
        SupraCommand::Main => {
            // Files
            let input = matches.value_of("input").unwrap();
            let library = matches.value_of("library").unwrap();
            let output = matches.value_of("output");
            let pandoc_reference = matches.value_of("pandoc_reference");

            // Pre-processor options
            let offset = *matches.get_one::<usize>("offset").unwrap() as i32;
            let user_journals = if matches.is_present("user_journals") {
                Some(matches.value_of("user_journals").unwrap())
            } else {
                None
            };
            let smallcaps = matches.is_present("smallcaps");
            let force_overwrite = matches.is_present("force_overwrite");

            // Post-processing options
            let autocref = matches.is_present("autocref");
            let author_note = matches.is_present("author_note");
            let tabbed_footnotes = matches.is_present("tabbed_footnotes");
            let no_superscript = matches.is_present("no_superscript");
            let running_header = matches.is_present("running_header");

            // Deal with command-line errors.
            //
            // If the input and output strings are identical and force_overwrite
            // has not been used, return an error and exit.
            if output.is_some() && input == output.unwrap() && !force_overwrite {
                eprintln!("{} The input file ({}) and output file ({}) are the same,\n     but the force overwrite option was not set.\n     If you want to overewrite the input file, use -W/--force_overwrite.", Color::Red.paint("ERRO"), Color::Blue.paint(input), Color::Blue.paint(output.unwrap()));
                process::exit(1);
            }

            // Determine the output
            let output_option = match output {
                Some(f) => {
                    if &f[f.len() - 3..] == ".md" {
                        Output::Markdown
                    } else if &f[f.len() - 5..] == ".docx" {
                        Output::Docx
                    } else {
                        eprintln!(
                            "{} The output file must have an .md or .docx extension. You used {}",
                            Color::Red.paint("ERRO"),
                            Color::Blue.paint(f)
                        );
                        process::exit(1);
                    }
                }
                None => Output::StandardOut,
            };

            // Create the configuration
            let pre_config = PreConfig::new(input, library, offset, user_journals, smallcaps);
            let pan_config = PanConfig::new(output, pandoc_reference);
            let post_config = PostConfig::new(
                autocref,
                author_note,
                tabbed_footnotes,
                no_superscript,
                running_header,
            );

            SupraConfig::new(
                command,
                Some(output_option),
                Some(pre_config),
                Some(pan_config),
                Some(post_config),
            )
        }
    };

    // Run the program.
    let _ = supra::supra(config);

    Ok(())
}

/// Tests a project name for valid characters.
///
/// New project names must consist only of characters that the file system is
/// okay with. This returns a bool on whether the name is valid.
fn valid_name(name: &str) -> bool {
    for c in name.chars() {
        if !valid_char(&c) {
            return false;
        }
    }
    true
}

/// Checks for invalid characters.
///
/// Takes a character and returns a bool of whether it's okay.
fn valid_char(x: &char) -> bool {
    !r#" #%&{}\<>*?/$!'":@+`|="#.chars().any(|y| y == *x)
}

#[cfg(test)]
mod tets {
    use super::*;

    #[test]
    fn all_valid() {
        let input = "test";
        assert!(valid_name(input));
    }

    #[test]
    fn not_valid() {
        let input = "test ";
        assert!(!valid_name(input));
    }

    #[test]
    fn good_chars() {
        let good_chars = [
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
            'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '1', '2', '3', '4', '5', '6', '7', '8',
            '9', '0', '(', ')', '.',
        ];
        for c in good_chars {
            assert!(valid_char(&c));
        }
    }

    #[test]
    fn bad_chars() {
        let bad_chars = [
            ' ', '#', '%', '&', '{', '}', '\\', '<', '>', '*', '?', '/', '$', '!', '\'', '"', ':',
            '@', '+', '`', '|', '=',
        ];
        for c in bad_chars {
            assert!(!valid_char(&c));
        }
    }
}
