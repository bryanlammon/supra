//! The main Supra function. This calls the pre-processor, Pandoc, and the
//! post-processor.

pub mod config;
mod fs;
mod pan;
mod post;
mod pre;

use crate::config::Output;
use ansi_term::Color;
use config::{SupraConfig, SupraSubcommand};
use fs::load_file;
use slog::{debug, error, o};
use std::{path::Path, process};

pub fn supra(config: SupraConfig) -> Result<(), String> {
    // Check supra_subcommands
    match config.supra_subcommand {
        Some(SupraSubcommand::NewUserJournalFile) => {
            debug!(slog_scope::logger(), "Creating blank user-journal file");
            fs::new_user_journals_ron();
            return Ok(());
        }
        Some(SupraSubcommand::NewProject) => {
            // TODO
        }
        None => (),
    }

    eprintln!("{} Starting Supra...", Color::Green.paint("INFO"));

    // Create paths fort the input, library, etc.
    let input = Path::new(config.pre_config.input);
    let library = Path::new(config.pre_config.library);
    let output = config.pan_config.output.map(Path::new);
    let pandoc_reference = config.pan_config.pandoc_reference.map(Path::new);

    // Load the input
    let input =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "load_file()")), || {
            load_file(input)
        }) {
            Ok(i) => i,
            Err(e) => {
                error!(slog_scope::logger(), "Markdown load error: {}", e);
                eprintln!("{} Markdown load error: {}", Color::Red.paint("ERRO"), e);
                process::exit(1);
            }
        };

    // Load the CSL JSON library
    let library =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "load_file()")), || {
            load_file(library)
        }) {
            Ok(l) => l,
            Err(e) => {
                error!(slog_scope::logger(), "CSL JSON load error: {}", e);
                eprintln!("{} CSL JSON load error: {}", Color::Red.paint("ERRO"), e);
                process::exit(1);
            }
        };

    // Load the user journals, if any
    let user_journals = match config.pre_config.user_journals {
        Some(u) => {
            match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "load_file()")), || {
                load_file(Path::new(&u))
            }) {
                Ok(j) => Some(j),
                Err(e) => {
                    error!(slog_scope::logger(), "User journals load error: {}", e);
                    eprintln!(
                        "{} User journals load error: {}",
                        Color::Red.paint("ERRO"),
                        e
                    );
                    process::exit(1);
                }
            }
        }
        None => None,
    };

    // Run the pre-processor
    eprintln!("{} Pre-processing...", Color::Green.paint("INFO"));

    let pre = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "pre()")), || {
        pre::pre(
            &input,
            &library,
            &user_journals,
            config.pre_config.offset,
            config.pre_config.smallcaps,
        )
    }) {
        Ok(p) => p,
        Err(e) => {
            error!(slog_scope::logger(), "Pre-processing error: {}", e);
            eprintln!("{} Pre-processing error: {}", Color::Red.paint("ERRO"), e);
            process::exit(1);
        }
    };

    // If no output or Markdown output were selected, output now
    if config.output == Output::StandardOut {
        println!("{}", pre);
        return Ok(());
    } else if config.output == Output::Markdown {
        // This can safely unwrap because an output must have been provided for
        // config.output to be set to Markdown
        return fs::save_file(output.unwrap(), &pre);
    }

    // Run Pandoc on the pre-processor output
    eprintln!("{} Running Pandoc...", Color::Green.paint("INFO"));

    // If running pandoc, there must be an output
    let output = output.unwrap();

    match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "pan()")), || {
        pan::pan(&pre, output, pandoc_reference)
    }) {
        Ok(o) => o,
        Err(e) => {
            error!(slog_scope::logger(), "Pandoc error: {:?}", e);
            eprintln!("{} Pandoc error: {:?}", Color::Red.paint("ERRO"), e);
            process::exit(1);
        }
    };

    // If any post-processing options are true, run the post-processor on the
    // Pandoc .docx output
    if config.post_config.autocref
        || config.post_config.author_note
        || config.post_config.tabbed_footnotes
        || config.post_config.no_superscript
        || config.post_config.running_header
    {
        eprintln!("{} Post-processing...", Color::Green.paint("INFO"));
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "post()")), || {
            post::post(
                &input,
                output,
                config.post_config.autocref,
                config.post_config.author_note,
                config.post_config.tabbed_footnotes,
                config.post_config.no_superscript,
                config.post_config.running_header,
            )
        }) {
            Ok(p) => p,
            Err(e) => {
                error!(slog_scope::logger(), "Post-processing error: {}", e);
                eprintln!("{} Post-processing error: {}", Color::Red.paint("ERRO"), e);
                process::exit(1);
            }
        };
    }

    eprintln!("{} Done", Color::Green.paint("INFO"));
    Ok(())
}
