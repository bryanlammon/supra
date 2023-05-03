//! Contains the main Supra function. Determines which parts of Supra to run.

pub mod config;
mod fs;
mod pan;
mod post;
pub mod pre;

use crate::config::Output;
use ansi_term::Color;
use config::{SupraCommand, SupraConfig};
use fs::load_file;
use slog::{debug, error, o};
use std::{path::Path, process};

/// The main Supra function.
pub fn supra(config: SupraConfig) -> Result<(), String> {
    // Check subcommands.
    match config.command {
        SupraCommand::NewUserJournalFile => {
            debug!(slog_scope::logger(), "Creating blank user-journal file");
            fs::new_user_journals_ron();
            return Ok(());
        }
        SupraCommand::NewProject(name, overwrite) => {
            debug!(slog_scope::logger(), "Creating new project");
            fs::new_project(name, overwrite);
            return Ok(());
        }
        SupraCommand::ReplaceMake => {
            debug!(slog_scope::logger(), "Replacing Makefile");
            fs::replace_make();
            return Ok(());
        }
        _ => {}
    }

    eprintln!("{} Starting Supra...", Color::Green.paint("INFO"));

    // Create paths fort the input, library, etc.
    let input = Path::new(config.pre_config.as_ref().unwrap().input);
    let library = Path::new(config.pre_config.as_ref().unwrap().library);
    let output = config.pan_config.as_ref().unwrap().output.map(Path::new);
    let pandoc_reference = config
        .pan_config
        .as_ref()
        .unwrap()
        .pandoc_reference
        .map(Path::new);

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
    let user_journals = match config.pre_config.as_ref().unwrap().user_journals {
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
            config.pre_config.as_ref().unwrap().offset,
            config.pre_config.as_ref().unwrap().smallcaps,
        )
    }) {
        Ok(p) => p,
        Err(e) => {
            error!(slog_scope::logger(), "Pre-processing error: {}", e);
            eprintln!("{} Pre-processing error: {}", Color::Red.paint("ERRO"), e);
            process::exit(1);
        }
    };

    // If no output or Markdown output was selected, output now
    if config.output.as_ref().unwrap() == &Output::StandardOut {
        println!("{}", pre);
        return Ok(());
    } else if config.output.as_ref().unwrap() == &Output::Markdown {
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
    if config.post_config.as_ref().unwrap().autocref
        || config.post_config.as_ref().unwrap().author_note
        || config.post_config.as_ref().unwrap().tabbed_footnotes
        || config.post_config.as_ref().unwrap().no_superscript
        || config.post_config.as_ref().unwrap().running_header
    {
        eprintln!("{} Post-processing...", Color::Green.paint("INFO"));
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "post()")), || {
            post::post(
                &input,
                output,
                config.post_config.as_ref().unwrap().autocref,
                config.post_config.as_ref().unwrap().author_note,
                config.post_config.as_ref().unwrap().tabbed_footnotes,
                config.post_config.as_ref().unwrap().no_superscript,
                config.post_config.as_ref().unwrap().running_header,
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
