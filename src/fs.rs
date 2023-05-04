//! Functions for interacting with the file system (except .docx).

mod file_contents;

use ansi_term::Color;
use file_contents::{BLANK_USER_JOURNAL_CONTENTS, MAKEFILE_CONTENTS, MD_CONTENTS};
use git2::Repository;
use slog::debug;
use std::{fs, path::Path};

/// Load a file into a string.
///
/// This function is used to load both the markdown and CSL JSON files into
/// strings, which can then be passed to the main function.
pub fn load_file(path: &Path) -> Result<String, String> {
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

/// Save a string in a file.
///
/// This function saves the provided string to a file. It is used when outputting to Markdown.
pub fn save_file(path: &Path, output: &str) -> Result<(), String> {
    debug!(slog_scope::logger(), "Saving {}...", path.to_string_lossy());
    eprintln!(
        "{} Saving {}...",
        Color::Green.paint("INFO"),
        Color::Blue.paint(path.to_string_lossy())
    );

    match std::fs::write(path, output) {
        Ok(_) => {
            debug!(
                slog_scope::logger(),
                "File {} saved.",
                path.to_string_lossy()
            );
            eprintln!("{} Done", Color::Green.paint("INFO"));
            Ok(())
        }
        Err(e) => {
            let err_msg = format!("Error writing the file {}—{}", path.to_string_lossy(), e);
            Err(err_msg)
        }
    }
}

/// Create a blank user-journals file.
///
/// Creats a blank user-journal sfile that users can then fill in with their own
/// journals.
pub fn new_user_journals_ron() {
    eprintln!(
        "{} Creating blank user-journal file ({})",
        Color::Green.paint("INFO"),
        Color::Blue.paint("blank-user-journals.ron")
    );

    fs::write("blank-user-journals.ron", BLANK_USER_JOURNAL_CONTENTS)
        .expect("Unable to write blank user-journals file");
}

/// Create a new project.
pub fn new_project(name: &str, git: bool, overwrite: bool) {
    eprintln!(
        "{} Creating new project {}",
        Color::Green.paint("INFO"),
        Color::Blue.paint(name)
    );

    // The filenames for a new project.
    let root = format!("./{}/", name);
    let src = format!("{}src/", root);
    let build = format!("{}build/", root);
    let makefile = format!("{}Makefile", root);
    let md = format!("{}/{}.md", src, name);

    // Try to create the project directory. If it exists, note a warning.
    if Path::new(&root).exists() {
        eprintln!(
            "{} Cannot create {}: Directory exists",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(&root)
        );
    } else if let Err(e) = fs::create_dir(&root) {
        eprintln!(
            "{} Error creating {}: {}",
            Color::Yellow.paint("ERRO"),
            Color::Red.paint(&root),
            e
        );
    };

    // Try to create the Makefile. If it exists and overwrite is not set, note a warning.
    if Path::new(&makefile).exists() && !overwrite {
        eprintln!(
            "{} Cannot create {}: File exists (use -W to force overwrite)",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(&makefile)
        )
    } else if let Err(e) = fs::write(&makefile, MAKEFILE_CONTENTS) {
        eprintln!(
            "{} Error creating {}: {}",
            Color::Red.paint("ERRO"),
            Color::Blue.paint(&makefile),
            e
        );
    }

    // Try to create the source directory. If it exists, note a warning.
    if Path::new(&src).exists() {
        eprintln!(
            "{} Cannot create {}: Directory exists",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(&src)
        );
    } else if let Err(e) = fs::create_dir(&src) {
        eprintln!(
            "{} Error creating {}: {}",
            Color::Red.paint("ERRO"),
            Color::Blue.paint(&src),
            e
        );
    }

    if Path::new(&md).exists() && !overwrite {
        eprintln!(
            "{} Cannot create {}: File exists (use -W to force overwrite)",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(&md)
        )
    } else if let Err(e) = fs::write(&md, MD_CONTENTS) {
        eprintln!(
            "{} Error creating {}: {}",
            Color::Red.paint("ERRO"),
            Color::Blue.paint(&md),
            e
        );
    }

    if Path::new(&build).exists() {
        eprintln!(
            "{} Cannot create {}: Directory exists",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(&build)
        );
    } else if let Err(e) = fs::create_dir(&build) {
        eprintln!(
            "{} Error creating {}: {}",
            Color::Red.paint("ERRO"),
            Color::Blue.paint(&build),
            e
        );
    }

    if git {
        if let Err(e) = Repository::init(Path::new(&root)) {
            eprintln!(
                "{} Error initiating git repository in {}: {}",
                Color::Red.paint("ERRO"),
                Color::Blue.paint(&root),
                e
            )
        }
    }
}

/// Replace the current directory's Makefile with the Supra Makefile.
pub fn replace_make() {
    eprintln!("{} Replacing Makefile", Color::Green.paint("INFO"));
    if let Err(e) = fs::write("./Makefile", MAKEFILE_CONTENTS) {
        eprintln!(
            "{} Error creating {}: {}",
            Color::Red.paint("ERRO"),
            Color::Blue.paint("./Makefile"),
            e
        );
    }
}
