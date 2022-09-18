# Supra <!-- omit in toc -->

Supra lets you write legal scholarship in Markdown.

- [About](#about)
- [Setup](#setup)
  - [Supra Markup](#supra-markup)
    - [Citations](#citations)
    - [Internal Cross-References](#internal-cross-references)
  - [Source Library](#source-library)
- [Usage & Options](#usage--options)
  - [Basic Usage](#basic-usage)
  - [Pre-Processor Options](#pre-processor-options)
    - [Small Caps](#small-caps)
    - [Offsetting](#offsetting)
    - [User Journal File](#user-journal-file)
    - [Overwriting the Input File](#overwriting-the-input-file)
  - [Pandoc Options](#pandoc-options)
  - [Post-Processing Options](#post-processing-options)
    - [Field Cross-References](#field-cross-references)
    - [Insert Author Note](#insert-author-note)
    - [Tabs After Footnotes](#tabs-after-footnotes)
    - [Non-Superscript Footnote Numbers](#non-superscript-footnote-numbers)
    - [Running Header](#running-header)
- [Recommended Project Organization](#recommended-project-organization)
  - [New Project](#new-project)
  - [Makefile](#makefile)
- [Changelog](#changelog)

## About

Supra is a [Pandoc](https://pandoc.org) wrapper for legal scholarship.
Pandoc is great for academic writing.
But there are several aspects of legal scholarship that Pandoc doesn't do well, particularly the innumerable footnotes with oodles of cross-references—*i.e.*, *supra* notes.

Supra makes Pandoc more useful for writing legal scholarship.
Its main feature is a pre-processor that (1) inserts citations for common source types that use cross-references, (2) adds cross-references for those sources in subsequent footnotes, and (3) adds cross-references among footnotes.
Supra can then call Pandoc with an optional custom reference.
Finally, a post-processor can edit the `.docx` file that Pandoc produces, turning footnote cross-references into fields (which can then update automatically), adding an author footnote, and more.

## Setup

At a bare minimum, Supra requires a file in Pandoc-markdown format—with [Supra's additional markup](#supra-markup)—and a source library in [CSL JSON](https://citationstyles.org) format.
The pre-processor can be used without [Pandoc](https://pandoc.org)—outputting to either standard out or a markdown file.
But Pandoc is necessary to get the most out of Supra.
And you should also use a Pandoc custom reference for the `.docx` output.
Supra come with two custom references.

### Supra Markup

Supra's main feature is processing citations and cross-references.
It searches the footnotes in a Pandoc document for certain kinds of sources (see [Supported Source Types](#source-library)).
Using a CSL JSON library, the pre-processor changes the first citation to the standard full citation form, following the [Indigo Book](https://law.resource.org/pub/us/code/blue/IndigoBook.html) guidelines.
It then changes all subsequent citations to the "*supra* note" format with the correct footnote number.
If multiple citations have the same author, Supra also adds a "hereinafter" to the long cite and a short title to subsequent cites.
Finally, Supra looks for cross-references in and to other footnotes.

To do all of this, Supra uses an extended (and slightly altered) version of [Pandoc's markdown](https://pandoc.org/MANUAL.html#pandocs-markdown), described below.

#### Citations

Citations must be (1) inside inline footnotes and (2) in the form of an ID that begins with `@` and is surrounded by brackets.
The ID is from your CSL JSON library.
A plugin for your text editor (like [CiteBibtex](https://packagecontrol.io/packages/CiteBibtex) for [Sublime Text](https://www.sublimetext.com)) is really useful for adding these.

Note, a document written with reference-style footnotes can be converted to inline footnotes using the [`inliner`](https://github.com/ltrgoddard/inliner) python script.

```Markdown
# A simple example
Some text.^[*See* [@Smith2004].]

# Another simple example
Some more text.^[For an in-depth discussion of the *Johnson*
case, see [@Jones2003].]
```

Each citation must be in its own pair of brackets.
Note, this is a break from Pandoc's markdown, which allows you to put multiple sources in a [single set of brackets](https://pandoc.org/MANUAL.html#citation-syntax).
But given the common style of legal scholarship, this break should not pose any problems.

```Markdown
# A multiple-cite example
Some text.^[*See* [@Smith2004]; [@Jones2004].]
```

Supra can also recognize pincites in various formats.
The pincite must come after the closing bracket for a citation.
Again, this is a break from Pandoc's markdown, which puts pincites [within the brackets](https://pandoc.org/MANUAL.html#citation-syntax).

```Markdown
# Some pincite examples
Some text.^[*See* [@Smith2004] 123.]

Some more text.^[*See also* [@Jones2003] at 123 n.4.]

Even more text.^[*See* [@Smith2004] 123; [@Williams] §
\3944.]

# You can use "tk" for unknown page numbers (e.g.,
# forthcoming articles)
I'm not sure what page I'm referring to yet.^[*But see*
[@Johnson2021] at tk.]
```

An "at" is optional, and Supra will ensure that citation types that require an "at" have one.

Note, Supra will change all subsequent citations to the `*supra* note #` format; it will not insert `*Id.*`s.
So if you cite the same source in subsequent footnotes, you must manually add the `*Id.*`s.

#### Internal Cross-References

Supra can also add cross-references to other footnotes.
This requires adding an ID to the referred-to footnote, which is a unique string that begins with a `?`, is surrounded by brackets, and is the first thing in the footnote.
The footnote can then be referred to with that ID.

Supra will not add *supra*, *infra*, or the word "note" to these cross-references.
There are too many ways of phrasing internal cross-references (*e.g.*, *see* *supra* note 1; *see* *supra* text accompanying notes 1–2; *see* *infra* notes 3 & 4 and accompanying text).
Supra doesn't know which phrasing you want.
So you must write the rest of the internal cross-reference yourself.

Note, if you want your footnote cross-references to be easily updated fields (using the [post-processor options](#field-cross-references)), ensure that there are no commas after *infra* or *supra*.
Otherwise, Supra will not recognize the cross-reference.

```Markdown
# A Footnote with a Cross-References
Some text.^[[?id1] This footnote has an ID. For another
footnote, see *infra* note [?id2].]

# Referring Back to a Footnote
Some more text.^[[?id2] For another footnote, see *supra*
note [?id1].]

# Referring Back to a Range of Footnotes
Even more text.^[*See* *supra* notes [?id1]–[?id2] and
accompanying text.]
```

### Source Library

Supra gets citation information from your source library, which must be in [CSL JSON](https://citationstyles.org) format.
One easy way to create and continually update a CSL JSON library is to use [Zotero](https://www.zotero.org) (for reference management) and the [Better BibTeX for Zotero](https://retorque.re/zotero-better-bibtex/) plugin (to automatically create and update the CSL JSON library).

Supra currently supports four source types:

* Books,
* Book chapters (*i.e.*, separately authored contributions to a collection),
* Consecutively paginated journal articles, and
* Unpublished manuscripts.

For books, book chapters, and consecutively paginated journal articles, Supra uses the expected CSL JSON fields.
If you have multiple sources from the same author in your library, you should add a short title to each source.
For unpublished manuscripts that are forthcoming in a law review, you can add `volume` and `container-title` fields to produce a citation in "forthcoming" format, *e.g.,* June Smith, *An Article About Someting*, 10 Law J. (forthcoming 2021).
In Zotero, you can enter those on separate lines in the "Extra" field:

```Markdown
container-title: Law Journal
volume: 10
```

There is also limited support for non-consecutively paginated journals, book reviews, student-written material, and treatises that have non-page-number pincites (e.g., § 1001).

## Usage & Options

Supra is a command-line program.
It operates in three stages: (1) a [Pandoc pre-processor](#pre-processor-options), (2) an [optional call to Pandoc](#pandoc-options), and (3) an [optional post-processor](#post-processing-options) for the Pandoc output.

### Basic Usage

Supra requires two arguments: the Pandoc-markdown file and the CSL JSON library.
These files are expected at positions one and two, respectively.
With only two arguments, the output will go to standard out, which can then be manually piped into Pandoc.

```sh
# Two-argument example
supra input.md library.json

# Piping example
supra input.md library.json | pandoc --from=markdown -o output.docx
```

An optional third argument is the output file.
That file must end with an `.md` or `.docx` extension.
An `.md` will output in Markdown format.
`.docx` will use Pandoc to output a Word document.

```sh
# Three-argument example with Markdown output
supra input.md library.json output.md

# Three-argument example with .docx output
supra input.md library.json output.docx
```

Finally, an optional fourth argument is the Pandoc [custom reference file](https://pandoc.org/MANUAL.html#option--reference-doc).
Invoking this argument requires that the third argument (the output file) end with a `.docx` extension.
If you output to Markdown, Pandoc would not be run and the custom reference would be useless.
If you did not supply an output file at all, Supra would output to the custom reference file.

```sh
# Using a custom reference
supra input.md library.json output.docx custom-reference.docx
```

### Pre-Processor Options

The pre-processor has a few additional options.

#### Small Caps

```sh
-s/--smallcaps
```

A Pandoc lua filter can set certain text to small caps (e.g., [bolded text to small caps](https://pandoc.org/lua-filters.html)).
If outputting to a docx file, however, the text is not [true small caps](https://en.wikipedia.org/wiki/Small_caps#Word_processors).

Supra includes an option to set bolded text to a Word style called "Small Caps."
That Word style can then apply true small caps via the appropriate typeface.
This option can be set with `-s` or `--smallcaps`.

This is useful only if the output docx file has a "Small Caps" style.
If using this option and a [custom reference file](https://pandoc.org/MANUAL.html#option--reference-doc) for Pandoc, you should add that style to the custom reference.
Both of Supra's supplied custom references include a "Small Caps" style (though the Century Schoolbook custom reference does not use true small caps; it uses Word's built-in small caps functionality).

#### Offsetting

```sh
-f/--offset <NUMBER>
```

Supra normally assumes that the first footnote in a document is numbered 1.
If you plan to later change the numbers for any footnotes in the Word document—say, to start at a later number—then you need to offset the footnote counter.
The offset counter is invoked with the `-f` or `--offset` argument.
The first footnote will be treated as the offset + 1.
So, for example, to treat the second footnote in the document as footnote 1, you would follow the argument with `-1`

```sh
# Skip the first footnote; all references to the second
# footnote in the document will call it note "1"
supra input.md library.json output.md -f -1
```

To start at footnote 100, you would follow the argument with `99`:

```sh
# Start footnote numbering at 100
supra input.md library.json output.md -f 99
```

I have a hard time imagining when a negative number would be useful.
You should not try to manually add in an author/star/asterisk note—that requires way too much fiddling with Word.
It's much easier to juse the [`-a/--author`](#insert-author-note) option.
But Supra allows you to use a negative offset if you really want to.

#### User Journal File

```sh
-u/--user_journals <FILE>
```

Supra has a built-in list of short names for several hundred law journals.
(See [`src/pre/sourcemap/buildsource/journalnames.rs`](https://github.com/bryanlammon/supra/blob/main/src/pre/sourcemap/buildsource/journalnames.rs), and feel free to contribute additional journals.)
It will also attempt to abbreviate journal names that it does not know using the Indigo Book guidelines, and you will be notified of these attempts when running Supra.

You can also supply your own collection of abbreviated journal names.
The names must be in the form of a Supra user-journal file.
You can create a blank user-journal file by running `supra uj`.
This will create a file called `blank-user-journals.ron`, which you can rename to whatever you want.
Open the file in any plain-text editor, and you will find instructions on how to add journals and a placeholder example.

To run Supra with a user-journal file, add the argument `-u` or `--user_journals`, follwed by the file name.

```sh
# Create a blank user-journal file
supra uj

# Use a custom user-journal file
supra input.md library.json output.docx -u my-journals.ron
```

#### Overwriting the Input File

```sh
-W/--force_overwrite
```

If outputting to Markdown, Supra will not automatically overwrite the input file.
If you try to output to the same file as the input file, Supra will return an error.

If you *really* want to overwrite the input file, you must use the option `-W` or `--force_overwrite`.

### Pandoc Options

Supra's second step is an optional use of Pandoc.
If you send Supra's output to the terminal or a Markdown file, neither Pandoc nor the post-processor will run.
But if you set your output to a `.docx` file, Supra will run the pre-processor's output through Pandoc, producing a `.docx` file.

As noted above in [Basic Usage](#basic-usage), Supra also allows the use of a [custom reference](https://pandoc.org/MANUAL.html#options-affecting-specific-writers).
Supra comes with two custom reference files, both of which work with all of Supra's options:

* [`supra-reference-cs.docx`](https://github.com/bryanlammon/supra/blob/main/supra-reference-cs.docx), which uses common legal scholarship formatting, and
* [`supra-reference-cormorant.docx`](https://github.com/bryanlammon/supra/blob/main/supra-reference-cormorant.docx), which uses my preferred formatting.

Note, use of `supra-reference-cormorant.docx` requires that you have the fonts Cormorant Garamond and Cormorant SC installed.
You can download them [here](https://github.com/CatharsisFonts/Cormorant).

### Post-Processing Options

Supra can also make some edits to the `.xml` markup in the `.docx` file that Pandoc produces.
These can make the journal-editing stage of law review writing easier, and they can reduce (or maybe even eliminate) the time that you must spend in Microsoft Word.

#### Field Cross-References

```sh
-c/--autocref
```

Turns the footnote cross-references in the `.docx` file into Microsoft Word fields, which can then be easily updated.

This is useful for the editing stages of legal scholarship.
The addition or subtraction of footnotes that often happens during editing can require updating the cross-referenced footnote numbers.
With automatically updating cross-references, you just need to tell Word to [update all fields](https://support.microsoft.com/en-us/office/update-fields-7339a049-cb0d-4d5a-8679-97c20c643d4e).

#### Insert Author Note

```sh
-a/--author
```

Adds an author note (sometimes called a star footnote or asterisk footnote) using metadata from the Pandoc file.

Given that author notes normally aren't numbered, I recommend against adding them directly in your Pandoc document.
You can instead add an `author_note` field to the [YAML metadata block](https://pandoc.org/MANUAL.html#extension-yaml_metadata_block) in your Pandoc document:

```yaml
---
title: The Article Title
author: Author's Name
author_note: A note about the author.
---
```

The post-processor will find the last word in the `author` field, add a star (*i.e.*, \*) footnote, and use the contents of `author_note` for that * footnote's contents.

Note, this option requires only a single `author` entry and a single note.
If there are multiple authors, they should all be entered as one author in the YAML metadata block for this option to work.
E.g.:

```yaml
---
title: The Article Title
author: Author One & Author Two
author_note: A note about Author One. And a note about Author Two.
---
```

#### Tabs After Footnotes

```sh
-t/--tabs
```

Replaces the spaces after footnote numbers with tabs.

In my article formatting (output using [`supra-reference-cormorant.docx`](#pandoc-options)), I prefer tabs rather than spaces after footnote numbers.
This option replaces the spaces after the numbers with tabs.
Note, this affects only the footnote markers at the bottom of the page; footnote markers in the body text are unchanged.

#### Non-Superscript Footnote Numbers

```sh
-n/--no_superscript
```

Puts footnote numbers on the baseline.

In my article formatting (output using [`supra-reference-cormorant.docx`](#pandoc-options)), I prefer footnote markers be on the basline rather than superscript.
This option puts them on the baseline.
Note, this affects only the footnote markers at the bottom of the page; footnote markers in the body text are unchanged.

#### Running Header

```sh
-r/--header
```

Adds the year and short title to the header.

Supra's custom references include running headers, with a short title on every page and the year of the draft on odd pages.
This option fills those placeholders in.
To use this option, you must add the necessary information to `year` and `running_header` fields to the [YAML metadata block](https://pandoc.org/MANUAL.html#extension-yaml_metadata_block) in your Pandoc document:

```yaml
---
title: The Article Title
author: Author's Name
year: 2022
running_header: Title
---
```

Note, you must use a compatible [custom reference](#pandoc-options) for this option to work.
Both of the custom references provided with Supra are compatible.

## Recommended Project Organization

Supra can work with whatever project organization you want to use.
But it facilitates a particular approach that I've found useful.

At the root of this organization is a folder for all current research projects.
Each project is then in its own folder that is named after the project.
Alongside all of the projects is a `./_build-tools/` folder that contains Supra, your CSL JSON library, and any Pandoc custom references.
And each project folder contains a [Makefile](https://www.gnu.org/software/make/manual/make.html) (for running Supra, discussed momentarily) and two sub-folders: `./src/`, containing the Markdown file, and `./build/`, which contains Supra's output.
Both the Markdown file and output file share the project's name.

For example:

```
📂 research
┣ 📂 _build-tools
┃ ┣ 📄 my-library.json
┃ ┣ 📄 supra
┃ ┗ 📄 supra-reference-cormorant.docx
┃
┣ 📂 project_1
┃ ┣ 📂 build
┃ ┃ ┗ 📄 project_1.docx
┃ ┃
┃ ┣ 📂 src
┃ ┃ ┗ 📄 project_1.md
┃ ┃
┃ ┗ 📄 Makefile
┃
┗ 📂 project_2
  ┣ 📂 build
  ┃ ┗ 📄 project_1.docx
  ┃
  ┣ 📂 src
  ┃ ┗ 📄 project_1.md
  ┃
  ┗ 📄 Makefile
```

### New Project

```sh
new
```

If you use this structure, you can create new research projects with Supra's `new` subcommand.
You just type `supra new <name>`, with `<name>` being the name for your project.
Supra then creates a directory with that name, the `./src/` and `./build/` subdirectories, and a placeholder `.md` file and `Makefile`.
The `.md` file already has a YAML metadata block ready to fille out.
It also provides a space to write an abstract, using the abstract formatting in Supra's provided Pandoc custom references.

Note, the name cannot contain any spaces or characters that shouldn't go in directory or file names.

### Makefile

Even if you don't use the above approach, it's still easy to use Supra via a [Makefile](https://www.gnu.org/software/make/manual/make.html).
And I highly recommend one.
That way you can keep your reference library and custom Pandoc reference separate from any one project and use them library for all projects.
Below is adapted from the Makefile I use.
The `.md` input and `.docx` output go in separate directories inside a project.
The Supra library and Pandoc custom reference reside in a `/_build-tools/` directory that sits one level up, next to all of the project folders.

```Makefile
.PHONY: docx

source_dir := ./src/
build_dir := ./build/

source_file := $(source_dir)input.md
supra_lib := ../_build-tools/my-library.json
docx_file := $(build_dir)output.docx
docx_reference := ../_build-tools/supra-reference-cormorant.docx

build_tools :=\
    $(docx_reference) \
    $(supra_lib)

$(docx_file): $(source_file) $(build_tools)
    supra \
    $(source_file) \
    $(supra_lib) \
    $(docx_file) \
    $(docx_reference) \
    -scatnr

docx: $(docx_file)
```

## Changelog

* 0.1.0: Initial release
* 0.1.1: Fixed readme & blank-user journal command
* 0.1.2: Updated documentation
* 0.2.0: Added Pandoc and post-processing functionality
