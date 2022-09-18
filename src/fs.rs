//! Functions for interacting with the file system (except .docx).

use ansi_term::Color;
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

    let blank_ron = r#"// Enter your own journal abbreviations into this document.
// All entries must come between the two curly brackets, which start and end the
// file. Each entry should include two quoted strings, separated by a colon. The
// first string is the full journal title. The second string is the
// abbreviation. Put each journal on a separate line, with commas after every
// line. Below is an example:
//
// {
//  "Journal of Stuff":"J. Stuff",
//  "Journal of More Stuff":"J. More Stuff",
// }
//
// There is also a placeholder example below. Feel free to replace that with
// your own journals.

{
    "Full Journal Name":"Abbreviated Name",
}
"#;

    fs::write("blank-user-journals.ron", blank_ron)
        .expect("Unable to write blank user-journals file");
}

/// Create a new project.
pub fn new_project(name: &str) {
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

    // The contents of the files.
    let makefile_contents = r#"# Supra Makefile

.PHONY: all docx docx_cs md

MAKEFLAGS += --silent

mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))

current_dir := $(notdir $(patsubst %/,%,$(dir $(mkfile_path))))
source_dir := ./src/
build_dir := ./build/

source_file := $(source_dir)$(current_dir).md
md_file := $(build_dir)$(current_dir).md
docx_file := $(build_dir)$(current_dir).docx
docx_file_cs :=$(build_dir)$(current_dir)-cs.docx

docx_reference_book := ../_build-tools/supra-custom-reference-book.docx
docx_reference_cs := ../_build-tools/supra-custom-reference-cs.docx
supra_lib = ../_build-tools/my-library.json

all: $(docx_file) $(docx_file_cs)

build_tools :=\
	Makefile \
	$(docx_reference_book) \
	$(docx_reference_cs) \
	$(supra_lib)

$(docx_file): $(source_file) $(build_tools)
	supra \
	$(source_file) \
	$(supra_lib) \
	$(docx_file) \
	$(docx_reference_book) \
	-scatnr

$(docx_file_cs): $(source_file) $(build_tools)
	supra \
	$(source_file) \
	$(supra_lib) \
	$(docx_file_cs) \
	$(docx_reference_cs) \
	-scatnr

$(md_file): $(source_file) $(build_tools)
	supra \
	$(source_file) \
	$(supra_lib) \
	$(md_file)

docx: $(docx_file)

docx_cs: $(docx_file_cs)

md: $(md_file)"#;

    let md_contents = r#"---
title:
author:
author_note:
year:
running_header:
...

:::{custom-style="Abstract Title"}
abstract
:::

:::{custom-style="Abstract First Paragraph"}
tk
:::

:::{custom-style="Abstract Text"}
tk
:::
"#;

    // Try to create each directory and file
    if let Err(e) = fs::create_dir(root) {
        eprintln!(
            "{} Error creating root diectory: {}",
            Color::Red.paint("ERRO"),
            e
        );
    };
    if let Err(e) = fs::create_dir(src) {
        eprintln!(
            "{} Error creating source diectory: {}",
            Color::Red.paint("ERRO"),
            e
        );
    }
    if let Err(e) = fs::create_dir(build) {
        eprintln!(
            "{} Error creating build diectory: {}",
            Color::Red.paint("ERRO"),
            e
        );
    }

    if let Err(e) = fs::write(makefile, makefile_contents) {
        eprintln!(
            "{} Error creating Makefile: {}",
            Color::Red.paint("ERRO"),
            e
        );
    }
    if let Err(e) = fs::write(md, md_contents) {
        eprintln!(
            "{} Error creating Markdown file: {}",
            Color::Red.paint("ERRO"),
            e
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_load_file {
        use super::*;

        #[test]
        fn test_load() {
            let file = "./tests/test.md";
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
