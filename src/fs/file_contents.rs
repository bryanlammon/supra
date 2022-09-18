//! Contains the constants for file contents.

/// Contents of the blank user-journal file.
pub static BLANK_USER_JOURNAL_CONTENTS: &str = r#"// Enter your own journal abbreviations into this document.
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

/// Contents of the Supra Makefile created with new projects.
pub static MAKEFILE_CONTENTS: &str = r#"# Supra Makefile

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

/// Conents of the Supra Markdown template created with new projects.
pub static MD_CONTENTS: &str = r#"---
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
