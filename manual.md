# Supra manual

- [Supra manual](#supra-manual)
  - [Introduction](#introduction)
  - [Setup](#setup)
    - [Setting Up the Terminal](#setting-up-the-terminal)
    - [Installing Pandoc](#installing-pandoc)
    - [Setting Up a Source Library](#setting-up-a-source-library)
      - [Supported Source Types](#supported-source-types)
      - [Zotero + Better BibTex](#zotero--better-bibtex)
  - [Markup](#markup)
    - [Citations](#citations)
      - [Pincites](#pincites)
      - [*Id.*](#id)
    - [Internal Cross-References](#internal-cross-references)
  - [Usage \& Options](#usage--options)
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
    - [New Project Subcommand](#new-project-subcommand)
    - [Makefile](#makefile)
      - [Replace Makefile Subcommand](#replace-makefile-subcommand)
      - [Sample Makefile](#sample-makefile)

## Introduction

Supra is a [Pandoc](https://pandoc.org) wrapper for writing legal scholarship in Markdown.
Pandoc is great for academic writing.
But there are several aspects of legal scholarship that Pandoc doesn't do well, particularly the innumerable footnotes with oodles of cross-references—*i.e.*, *supra* notes.

Supra makes Pandoc more useful for writing legal scholarship.
Its main feature is a pre-processor that (1) inserts citations for common source types that use cross-references, (2) adds cross-references for those sources in subsequent footnotes, and (3) adds cross-references among footnotes.
Supra can then call Pandoc with an optional custom reference.
Finally, a post-processor can edit the `.docx` file that Pandoc produces, turning footnote cross-references into fields (which can then update automatically), adding an author footnote, and more.

## Setup

Before running Supra, you'll need a few things:

1. [Pandoc](https://pandoc.org);
2. A source library in [CSL JSON](https://citationstyles.org) format;
3. A plain-text file with the appropriate [Pandoc](https://pandoc.org/MANUAL.html#pandocs-markdown) and Supra markup; and
4. A Pandoc custom reference file.

Supra will techincally run with just 2 and 3, which will output the results to the terminal or another plain-text file.
But the rest is needed to get the most out of Supra.

### Setting Up the Terminal

Supra (like Pandoc) is a command-line program that runs in the terminal (you know, that thing that looks like MS-DOS).
To make your life easier, I recommend setting up a directory for your research in an easy-to-access location.
Then add a directory called `_build-tools`, which is where you will store the things you need to run Supra.
(You should add this `_build-tools` directory to your terminal path, which basically means that files in that folder will be accessible from every directory; search "add to path" and your operating system in your favorite search engine to learn more.)
Then use a separate folder for each project.

Note, you can use Supra to create a default directory for new projects—see the [new project subcommand](#new-project-subcommand) for more.

For more on project organization, see [Recommended Project Organization](#recommended-project-organization) below.

### Installing Pandoc

Instructions for installing Pandoc are available on [Pandoc's website](https://pandoc.org/installing.html) for a variety of operating systems.

### Setting Up a Source Library

Your source library is a file that contains all the information for sources that Supra will add to your documents.
It must contain all the information necessary to cite a source.

#### Supported Source Types

Supra currently supports five source types:

* Books,
* Book chapters (*i.e.*, separately authored contributions to a collection),
* Cases
* Consecutively paginated journal articles, and
* Unpublished manuscripts.

For books, book chapters, and consecutively paginated journal articles, Supra uses the expected CSL JSON fields.
(Note, you can provide an abbreviated journal name using the `container-title-short` field.)
If you have multiple sources from the same author in your library, you should add a short title to each source for potential "*hereinafter*" use.
And for unpublished manuscripts that are forthcoming in a law review, you can add `volume` and `container-title` fields to produce a citation in "forthcoming" format, *e.g.,* June Smith, *An Article About Someting*, 10 Law J. (forthcoming 2023).
In Zotero, you can enter those on separate lines in the "Extra" field:

```Markdown
container-title: Law Journal
volume: 10
```

For cases, Supra uses exactly what you enter into Zotero.
That means it does not (yet) abbreviate case names or check for the correct formatting of courts.

#### Zotero + Better BibTex

Probably the easiest way to set up a source library is by to use [Zotero](https://www.zotero.org) (for reference management) and the [Better BibTeX for Zotero](https://retorque.re/zotero-better-bibtex/) plugin (to automatically create and update the source library).
Instructions for installing Zotero are available on [Zotero's website](https://www.zotero.org/support/installation).
After installing Zotero, you can then install the Better BibTeX for Zotero plugin by following the instructions on the [plugin's website](https://retorque.re/zotero-better-bibtex/installation/).

The first thing you should do is set the Better BibTex plugin to automatically export your source library in CSL JSON format to your `_build-tools` directory.
Once again, Better BibTex's website has [instructions](https://retorque.re/zotero-better-bibtex/exporting/auto/) on how to setup automatic exports.

The next step is to add your sources to Zotero.
Everything you need to know is covered in Zotero's [documentation](https://www.zotero.org/support/).

## Markup

Supra's main feature is processing citations and cross-references.
It searches the footnotes in a Pandoc document for certain kinds of sources (see [Supported Source Types](#supported-source-types)).
Using a CSL JSON library, the pre-processor changes the first citation to the standard full citation form, following the [Indigo Book](https://law.resource.org/pub/us/code/blue/IndigoBook.html) guidelines.

Supra then uses the appropriate short form for the supported source.
If the source was just cited (and not cited as part of a string cite), Supra will use an *Id.*
If the source is a case cited in the last five footnotes, Supra will use the case's short form.
If the case hasn't been recently cited, Supra will use the long form.
And for books, articles, and the like, Supra changes subsequent citations to the "*supra* note" format with the correct footnote number (*e.g.*, "Jones, *supra* note 10, at 100").
If multiple citations have the same author, Supra also adds a "hereinafter" to the long cite and a short title to subsequent cites.
Finally, Supra looks for cross-references in and to other footnotes.

To do all of this, Supra uses a slightly extended (and slightly altered) version of [Pandoc's markdown](https://pandoc.org/MANUAL.html#pandocs-markdown), described below.
So user's should be familiar with Pandoc's markdown.
For most legal scholarship, only a few parts of that markdown will be relevant:

* [Paragraphs](https://www.pandoc.org/MANUAL.html#paragraphs)
* [ATX-style headings](https://www.pandoc.org/MANUAL.html#atx-style-headings)
* [Block quotations](https://www.pandoc.org/MANUAL.html#block-quotations)
* [Lists](https://www.pandoc.org/MANUAL.html#lists)
* [Emphasis](https://www.pandoc.org/MANUAL.html#extension-yaml_metadata_block)
* [Inline footnotes](https://www.pandoc.org/MANUAL.html#extension-inline_notes)

You should also have some idea of what a Pandoc YAML [metadata block is](https://www.pandoc.org/MANUAL.html#extension-yaml_metadata_block).

All that's left is Supra's markup, described below.

### Citations

Citations must be (1) inside inline footnotes and (2) in the form of an ID that begins with `@` and is surrounded by brackets.
The ID is from your CSL JSON library.
A plugin for your text editor (like [CiteBibtex](https://packagecontrol.io/packages/CiteBibtex) for [Sublime Text](https://www.sublimetext.com)) is really useful for adding these.

Note, a document written with reference-style footnotes can be converted to inline footnotes using the [`inliner`](https://github.com/ltrgoddard/inliner) python script.

```Markdown
# A simple example
Some text.^[*See* [@Smith2004].]

# Another simple example
Some more text.^[For an in-depth discussion of the *Johnson* case, see [@Jones2003].]
```

Each citation must be in its own pair of brackets.
Note, this is a break from Pandoc's markdown, which allows you to put multiple sources in a [single set of brackets](https://pandoc.org/MANUAL.html#citation-syntax).
But given the common style of legal scholarship, this break should not pose any problems.

```Markdown
# A multiple-cite example
Some text.^[*See* [@Smith2004]; [@Jones2004]; *see also* [@Williams1990].]
```

#### Pincites

Supra can recognize pincites in various formats.
The pincite must come after the closing bracket for a citation.
Again, this is a break from Pandoc's markdown, which puts pincites [within the brackets](https://pandoc.org/MANUAL.html#citation-syntax).

```Markdown
# Some pincite examples
Some text.^[*See* [@Smith2004] 123.]

Some more text.^[*See also* [@Jones2003] at 123 n.4.]

Even more text.^[*See* [@Smith2004] 123; [@Williams1990] § \ 3944.]

# You can use "tk" for unknown page numbers (e.g., forthcoming articles)
I'm not sure what page I'm referring to yet.^[*But see* [@Johnson2021] at tk.]
```

An "at" is optional, and Supra will ensure that citation types that require an "at" have one.

#### *Id.*

If you cite the same source twice (or more) in a row, Supra will change the citations to `*Id.*`
That way you don't need to concern yourself with short citations when you might move text around later.
Supra will also recognize when a source was previously cited as part of a string cite and not use *Id.*

If you cite to something that is not in your source library, you'll need to use a "cite breaker" to tell Supra that something else has been cited.
This is important for making sure that *Id.* works correctly.
A cite breaker is merely a dollar sign (`$`) surrounded by brackets (`[` and `]`) placed before the citation to a source not in your library.

```Markdown
Some text.^[*See* [@smith2004] at 123.]
Some more text.^[*See* [$] Plaintiff v. Defendant, 1 U.S. 1 (2000).]
Some more text.^[*See* [@smith2004] at 124.]
```

Without the cite breaker before the second cite in the example above, the third citation would have been rendered as an *Id.*

### Internal Cross-References

Supra can also add cross-references to other footnotes.
This requires adding an ID to the referred-to footnote, which is a unique string that (1) begins with a `?`, (2) is surrounded by brackets, and (3) is the first thing in the footnote.
The footnote can then be referred to with that ID.

```Markdown
# A Footnote with a Cross-References
Some text.^[[?id1] This footnote has an ID. For another footnote, see *infra* note [?id2].]

# Referring Back to a Footnote
Some more text.^[[?id2] For another footnote, see *supra* note [?id1].]

# Referring Back to a Range of Footnotes
Even more text.^[*See* *supra* notes [?id1]–[?id2] and accompanying text.]
```

Supra will not add *supra*, *infra*, or the word "note" to these cross-references.
There are too many ways of phrasing internal cross-references (*e.g.*, *see* *supra* note 1; *see* *supra* text accompanying notes 1–2; *see* *infra* notes 3 & 4 and accompanying text).
Supra doesn't know which phrasing you want.
So it will replace the IDs with a number.
You must write the rest of the internal cross-reference yourself.

Note, if you want your footnote cross-references to be easily updated fields (using the [post-processor options](#field-cross-references)), ensure that there are no commas after *infra* or *supra*.
Otherwise, Supra will not recognize the cross-reference.

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
An `.md` will output in Markdown format; `.docx` will run Pandoc to output a Word document.

```sh
# Three-argument example with Markdown output
supra input.md library.json output.md

# Three-argument example with .docx output
supra input.md library.json output.docx
```

Finally, an optional fourth argument is the Pandoc [custom reference file](https://pandoc.org/MANUAL.html#option--reference-doc).
Invoking this argument requires that the third argument (the output file) end with a `.docx` extension.
If you output to Markdown, Pandoc will not run and the custom reference will be useless.
And if you do not supply an output file at all—meaning that the custom reference file is your third argument—Supra will think that the custom reference file is your desired output.

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

A Pandoc Lua filter can set certain text to small caps (e.g., [bolded text to small caps](https://pandoc.org/lua-filters.html)).
If outputting to a docx file, however, the text is not [true small caps](https://en.wikipedia.org/wiki/Small_caps#Word_processors).

Supra includes an option to set bolded text to a Word style called "Small Caps."
That Word style can then apply true small caps via the appropriate typeface.
This option can be set with `-s` or `--smallcaps`.

This is useful only if the output docx file has a "Small Caps" style.
So a [custom reference file](https://pandoc.org/MANUAL.html#option--reference-doc) that includes this style is necessary.
Both of Supra's supplied custom references include a "Small Caps" style (though the Century Schoolbook custom reference does not use true small caps; it uses Word's built-in small caps functionality).

#### Offsetting

```sh
-f/--offset <NUMBER>
```

Supra normally assumes that the first footnote in a document is numbered 1.
If you plan to later change the numbers for any footnotes in the Word document—say, to start at a later number—then you need to offset the footnote counter.
The offset counter is invoked with the `-f` or `--offset` argument.
The first footnote will be treated as the provided offset + 1.
So, for example, to treat the second footnote in the document as footnote 1, you would follow the argument with `-1`

```sh
# Skip the first footnote; all references to the second footnote in the
# document will call it note "1"
supra input.md library.json output.md -f -1
```

To start at footnote 100, you would follow the argument with `99`:

```sh
# Start footnote numbering at 100
supra input.md library.json output.md -f 99
```

I have a hard time imagining when a negative number would be useful.
You should not try to manually add in an author/star/asterisk note—that requires way too much fiddling with Word.
It's much easier to use the [`-a/--author`](#insert-author-note) option discussed below.
But Supra allows you to use a negative offset if you really want to.

#### User Journal File

```sh
-u/--user_journals <FILE>
```

Supra has a built-in list of short names for several hundred law journals.
(See [`src/pre/sourcemap/buildsource/journalnames.rs`](src/pre/sourcemap/buildsource/journalnames.rs), and feel free to contribute additional journals.)
It will also read the `container-title-short` field in a CSL JSON file and use that over anything else.
And it will attempt to abbreviate journal names that it does not know using the Indigo Book guidelines; you will be notified of these attempts when running Supra.

You can also supply your own collection of abbreviated journal names.
The names must be in the form of a Supra user-journal file.
You can create a blank user-journal file by running `supra uj`.
This will create a file called `blank-user-journals.ron`, which you can rename to whatever you want.
Open the file in any plain-text editor, and you will find instructions on how to add journals.

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

As noted above in [Basic Usage](#basic-usage), Supra allows (and I encourage) the use of a [custom reference](https://pandoc.org/MANUAL.html#options-affecting-specific-writers).
Supra comes with a few custom reference files, all of which work with all of Supra's options:

* [`supra-reference-cs.docx`](https://github.com/bryanlammon/supra/blob/main/supra-reference-cs.docx), which uses common legal scholarship formatting;
* [`supra-reference-eb-garamond.docx`](https://github.com/bryanlammon/supra/blob/main/supra-reference-eb-garamond.docx), which uses my preferred formatting; and
* [`supra-reference-eb-garamond-wide.docx`](https://github.com/bryanlammon/supra/blob/main/supra-reference-eb-garamond-wide.docx), which uses my preferred formatting but adds wide margins; and

Both are formatted to look similar to published law review articles.
`supra-reference-cs.docx` is typeset in Century Schoolbook, and it has the wide margins you often see in PDFs of published articles.
It's a good default option.
`supra-reference-eb-garamond.docx` is my preference.
It is styled a little differently and uses narrow margins for easy reading on a tablet.
It also requires that you have the fonts EB Garamond and EB Garamond SC installed.
You can download those [here](https://github.com/georgd/EB-Garamond).

### Post-Processing Options

Supra can also make some edits to the `.docx` file that Pandoc produces.
These can make the journal-editing stage of law review writing easier, and they can reduce (and maybe even eliminate) the time that you must spend in Microsoft Word.

#### Field Cross-References

```sh
-c/--autocref
```

Turns footnote cross-references into Microsoft Word fields, which can then be easily updated.

This is useful for the editing stages of legal scholarship.
The addition and subtraction of footnotes that often happens during editing can require updating cross-referenced footnote numbers.
With fields, you simply need to tell Word to [update all fields](https://support.microsoft.com/en-us/office/update-fields-7339a049-cb0d-4d5a-8679-97c20c643d4e#_updateallfields).

#### Insert Author Note

```sh
-a/--author
```

Adds an author note (sometimes called a star footnote or asterisk footnote) using metadata from the Pandoc file.

Given that author notes normally aren't numbered, I recommend not adding them directly in your Pandoc document.
You can instead add an `author_note` field to the [YAML metadata block](https://pandoc.org/MANUAL.html#extension-yaml_metadata_block) in your Pandoc document:

```yaml
---
title: The Article Title
author: Author Name
author_note: A note about the author.
---
```

The post-processor will add a star (*i.e.*, \*) footnote after the last word in the `author` field and use the contents of `author_note` for that footnote's contents.

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

In my article formatting, I prefer tabs rather than spaces after footnote numbers.
This option replaces the spaces after the numbers with tabs.
Note, this affects only the footnote markers at the bottom of the page; footnote markers in the body text are unchanged.

#### Non-Superscript Footnote Numbers

```sh
-n/--no_superscript
```

Puts footnote numbers on the baseline.

In my article formatting, I prefer footnote markers be on the basline rather than superscript.
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
author: Author Name
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
Alongside all of the projects is a `_build-tools/` folder that contains Supra, your CSL JSON library, and Supra's custom references.
Each project folder contains a [Makefile](https://www.gnu.org/software/make/manual/make.html) (for running Supra, discussed momentarily) and two sub-folders: `src/`, containing the Markdown file, and `build/`, which contains Supra's output.
Both the Markdown file and output file share the project's name.

For example:

```
📂 research
┣ 📂 _build-tools
┃ ┣ 📄 my-library.json
┃ ┣ 📄 supra
┃ ┗ 📄 supra-reference-eb-garamond.docx
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

### New Project Subcommand

```sh
new
```

If you use this structure, you can create new research projects with Supra's `new` subcommand.
You just type `supra new <name>`, with `<name>` being the name for your project.
Supra then creates a directory with that name, the `src/` and `build/` subdirectories, and a placeholder `.md` file and `Makefile`.
The `.md` file already has a YAML metadata block ready to fill out.
It also provides a space to write an abstract, using the abstract formatting in Supra's provided Pandoc custom references.

```sh
# Make a new project called article
supra new article
```

Three notes about the `new` subcommand.
First, the name cannot contain any spaces or characters that shouldn't go in directory or file names.
Second, the default Supra Makefile expects both of Supra's custom references to be in your `_build-tools/` folder.
If one or both is missing, you'll need to edit the Makefile.
And third, by default, Supra will not overwrite your Markdown file or Makefile.
If you *really* want to overwrite the existing files, add the option `-W` or `--force_overwrite`.

```sh
# Overwrite the Markdown file and Makefile in the article folder
supra new article -W
```

### Makefile

Even if you don't use the above approach, I still recommend running Supra via a [Makefile](https://www.gnu.org/software/make/manual/make.html).
That way you can keep your reference library and custom Pandoc reference separate from any one project and use them for all projects.

#### Replace Makefile Subcommand

```sh
rmake
```

If you ever want to replace a Makefile in the current directory with Supra's default Makefile, you can run the subcommand `rmake`.

```sh
# Overwrite the Makefile in the current directory
supra rmake
```

#### Sample Makefile

If you don't use Supra's model project structure, you can still write your own Makefile.
Below is adapted from the one I use.
The `.md` input and `.docx` output go in separate directories inside a project.
The Supra library and Pandoc custom reference reside in a `/_build-tools/` directory that sits one level up, next to all of the project folders.

```Makefile
.PHONY: docx

source_dir := ./src/
build_dir := ./build/

source_file := $(source_dir)input.md
supra_lib := ../_build-tools/my-library.json
docx_file := $(build_dir)output.docx
docx_reference := ../_build-tools/supra-reference-cs.docx

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
