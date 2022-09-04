# Supra <!-- omit in toc -->

Supra is a [Pandoc](https://pandoc.org) preprocessor for writing U.S. legal scholarship.

- [About](#about)
- [Requirements](#requirements)
- [Setup](#setup)
  - [Supra Markup](#supra-markup)
    - [Citations](#citations)
    - [Internal Cross-References](#internal-cross-references)
  - [Supported Source Types & CSL JSON](#supported-source-types--csl-json)
- [Usage](#usage)
- [Additional Features](#additional-features)
  - [Small Caps](#small-caps)
  - [Offsetting](#offsetting)
  - [User Journal File](#user-journal-file)
  - [Overwriting the Input File](#overwriting-the-input-file)
- [Makefile](#makefile)
- [Changelog](#changelog)

## About

Supra is a [Pandoc](https://pandoc.org) preprocessor for writing U.S. legal scholarship.
Pandoc is great for academic writing.
But Pandoc doesn't have a way to to deal with the common format of U.S. legal scholarship, which normally contains innumerable footnotes with a plethora of cross-references—*i.e.*, *supra* notes.

Supra makes Pandoc more useful for writing U.S. legal scholarship by (1) inserting citations for common source types that use cross-references, (2) adding cross-references for those sources in subsequent footnotes, and (3) adding cross-references among footnotes.
More specifically, Supra searches the footnotes in a Pandoc markdown document for certain kinds of sources (see [Supported Source Types](#supported-source-types-csl-json) below).
Using a CSL JSON library, Supra changes the first citation to the standard full citation form.
It then changes all subsequent citations to the "*supra* note" format with the correct footnote number.
If multiple citations have the same author, Supra also adds a "hereinafter" to the long cite and a short title to subsequent cites.
Finally, Supra looks for cross-references in and to other footnotes.
All formatting follows the guidelines of the [Indigo Book](https://law.resource.org/pub/us/code/blue/IndigoBook.html).

## Requirements

Supra requires a file in [Pandoc](https://pandoc.org)-markdown format and a source library in [CSL JSON](https://citationstyles.org) format.
One easy way to create and continually update a CSL JSON library is to use [Zotero](https://www.zotero.org) (for reference management) and the [Better BibTeX for Zotero](https://retorque.re/zotero-better-bibtex/) plugin (to automatically create and update the CSL JSON library).

## Setup

### Supra Markup

Supra looks through Pandoc markdown documents for two things: citations in Pandoc format and footnote IDs.

#### Citations

Citations must be (1) inside inline footnotes and (2) in the form of an ID that begins with `@` and is surrounded by brackets.
The ID is from your CSL JSON library.

Note, a document written with reference-style footnotes can be converted to inline footnotes using the [`inliner`](https://github.com/ltrgoddard/inliner) python script.

```Markdown
# A simple example
Some text.^[*See* [@Smith2004].]

# Another simple example
Some more text.^[For an in-depth discussion of the *Johnson* case, see [@Jones2003].]
```

Each citation must be in its own pair of brackets.

```Markdown
# A multiple-cite example
Some text.^[*See* [@Smith2004]; [@Jones2004].]
```

Supra can also recognize pincites in various formats.

```Markdown
# Some pincite examples
Some text.^[*See* [@Smith2004] 123.]

Some more text.^[*See also* [@Jones2003] at 123 n.4.]

Even more text.^[*See* [@Williams] §\ 3944.]

# Use "tk" for unknown page numbers (e.g., forthcoming articles)
I'm not sure what page I'm referring to yet.^[*But see* [@Johnson2021] at tk.]
```

An "at" is optional, and Supra will ensure that citation types that require an "at" will have one.

#### Internal Cross-References

Supra can also add cross-reference to other footnotes.
This requires adding an ID to the referred-to footnote, which is a unique string that begins with a `?`, is surrounded by brackets, and is the first thing in the footnote.
The footnote can then be referred to with that ID.

Supra will not add *supra*, *infra*, or the word "note" to cross-references.
Supra can't tell whether the cross-references come before or after the ID'd footnote, so it doesn't know whether *supra* or *infra* is appropriate.
And there are many ways of phrasing internal cross-references (*e.g.*, *see* *supra* note 1; *see* *supra* text accompanying notes 1–2; *see* *infra* notes 3 & 4 and accompanying text).
Supra doesn't know which phrasing you want.
So you must write the rest of the internal cross-reference yourself.

Note, if you use [AutoCref](https://github.com/bryanlammon/autocref) for post-processing, ensure that there are no commas after *infra* or *supra*.
Otherwise, AutoCref will not recognize the cross-reference.

```Markdown
# A Footnote with a Cross-References
Some text.^[[?id1] This footnote has an ID.]

# Referring Back to a Footnote
Some more text.^[[?id2] For another footnote, see *supra* note [?id1].]

# Referring Back to a Range of Footnotes
Even more text.^[*See* *supra* notes [?id1]–[?id2] and accompanying text.]
```

### Supported Source Types & CSL JSON

Supra currently supports four source types:

* Books,
* Book chapters (*i.e.*, separately authored contributions to a collection),
* Consecutively paginated journal articles, and
* Unpublished manuscripts.

For books, book chapters, and consecutively paginated journal articles, Supra uses the expected CSL JSON fields.
For manuscripts, you can add `volume` and `container-title` fields to produce a citation in "forthcoming" format, *e.g.,* June Smith, *An Article About Someting*, 10 Law J. (forthcoming 2021).
In Zotero, you can enter those on separate lines in the "Extra" field:

```Markdown
container-title: Law Journal
volume: 10
```

There is also limited support for non-consecutively paginated journals, book reviews, student-written material, and treatises that have non-page-number pincites (e.g., §\ 1001).

## Usage

Supra is a command-line program.
It requires two arguments: the Pandoc-markdown file and the CSL JSON library.
These files are expected at positions one and two, respectively.
But they can also be set via `-i` or `--input` (for the input file) and `-l` or `--library` (for the library file).
With only two arguments, the output will go to standard out, which can then be piped into Pandoc.

```zsh
# Two-argument example
supra input.md library.json

# Piping example
supra input.md library.json | pandoc --from=markdown -o output.docx
```

An optional third argument is the output file.
This is expected as argument three, though it can also be set via `-o` or `--output`.

```zsh
# Three-argument example
supra input.md library.json output.md
```

## Additional Features

### Small Caps

A Pandoc lua filter can set certain text to small caps (e.g., [bolded text to small caps](https://pandoc.org/lua-filters.html)).
If outputting to a docx file, however, the text is not [true small caps](https://en.wikipedia.org/wiki/Small_caps#Word_processors).

Supra includes a flag to set bolded text to a Word style called "True Small Caps."
That Word style can then apply true small caps via the appropriate font.
This flag can be set with `-s` or `--smallcaps`.
This is useful only if the output docx file has a "True Small Caps" style.
If using this flag and a [custom reference file](https://pandoc.org/MANUAL.html#option--reference-doc) for Pandoc, you should add that style to the custom reference.

### Offsetting

Supra normally assumes that the first footnote in a document is numbered 1, the second 2, etc.
If you plan to change the numbers for any footnotes—say, to turn the first footnote into a \*—then you need to offset the footnote counter.
The offset counter is invoked with the `-f` or `--offset` argument.
To skip some of the first footnotes, follow the argument with a negative number.
To start at a later number, follow the argument with a positive number.

```zsh
# Skip the first footnote
supra input.md library.json output.md -f -1

# Start footnote numbering at 100
supra input.md library.json output.md -f 99
```

I'll note, however, that it's often easier to manually insert a \* footnote after the document is output to Word.

### User Journal File

Supra has a built-in list of abbreviations for about 400 common law journals.
(See [`src/sourcemap/buildsource/journalnames.rs`](https://github.com/bryanlammon/supra/blob/main/src/sourcemap/buildsource/journalnames.rs).)
It will also attempt to abbreviate journal names that it does not know, and you will be notified of these attempts when running Supra.

You can also supply your own collection of abbreviated journal names.
The names must be in the form of a user-journal file.
You can create a blank user-journal file by running `supra uj`.
This will create a file called `blank-user-journals.ron`.
Open the file in any plain-text editor, and you will find instructions on how to add journals and a placeholder example.

To run Supra with a user-journal file, add the argument `-u` or `--user_journals`, follwed by the file name.

```zsh
# Create a blank user-journal file
supra uj

# Use a custom user-journal file
supra input.md library.json output.md -u my-journals.ron
```

### Overwriting the Input File

By default, Supra will not overwrite the input file.
If you try to output to a file with the same name as the input file, Supra will return an error.
You normally must output to a different file name (or to standard out).

If you *really* want to overwrite the input file, you must add the flag `-W` or `--force_overwrite`.

## Makefile

The easiest way to use Supra is via a [Makefile](https://www.gnu.org/software/make/manual/make.html).
That way you can keep your reference library separate from any one project and use that library for all projects.
For example:

```Makefile
.PHONY: docx

source_dir := ./src/
build_dir := ./build/

source_file := $(source_dir)input.md
docx_file := $(build_dir)output.docx

docx_reference := ../_build-tools/pandoc-custom-reference.docx
supra_lib := ../_build-tools/my-library.json

build_tools :=\
    $(docx_reference) \
    $(supra_lib)

$(docx_file): $(source_file) $(build_tools)
    supra \
    $(source_file) \
    $(supra_lib) |\
    pandoc \
    --from=markdown \
    --reference-doc=$(docx_reference_book) \
    -o $(docx_file)

docx: $(docx_file)
```

## Changelog

* 0.1.0: Initial release
* 0.1.1: Fixed readme & blank-user journal command
* 0.1.2: Typo in documentation
