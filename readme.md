# Supra <!-- omit in toc -->

Supra lets you write legal scholarship in Markdown.

- [About](#about)
- [Setup \& Usage](#setup--usage)
- [Changelog](#changelog)

## About

Supra is a [Pandoc](https://pandoc.org) wrapper for writing legal scholarship in Markdown.
Pandoc is great for academic writing.
But there are several aspects of legal scholarship that Pandoc doesn't do well, particularly the innumerable footnotes with oodles of cross-references—*i.e.*, *supra* notes.

Supra makes Pandoc more useful for writing legal scholarship.
Its main feature is a pre-processor that (1) inserts citations for common source types that use cross-references; (2) where appropriate, adds short-form citations to those sources in subsequent footnotes; and (3) adds cross-references among footnotes.
Supra can then call Pandoc with an optional custom reference.
Finally, a post-processor can edit the file that Pandoc produces, turning footnote cross-references into fields (which can then update automatically), adding an author footnote, and more.

## Setup & Usage

For instructions on how to setup Supra, markup a document, and run the program, see the [manual](https://github.com/bryanlammon/supra/blob/main/manual.md).

## Changelog

* 0.1.0: Initial release
* 0.1.1: Fixed readme & blank-user journal command
* 0.1.2: Updated documentation
* 0.2.0: Added Pandoc and post-processing functionality
* 0.3.0: Added support for cases and automatic `*Id.*`s.
