//! This module contains the parser that prepares the tokens for markup.

use super::lexer::{Token, TokenType};
use slog::{debug, o, trace};

/// The types of branches in the syntax tree.
#[derive(Debug, PartialEq, Eq)]
pub enum Branch<'a> {
    Text(Text<'a>),
    FootnoteRef(FootnoteRef<'a>),
    CrossRef(CrossRef),
}

/// Contents of a text branch.
#[derive(Debug, PartialEq, Eq)]
pub struct Text<'a> {
    pub contents: &'a str,
}

impl Text<'_> {
    /// Create a new [`Text`] branch.
    fn new(contents: &str) -> Text {
        Text { contents }
    }
}

/// Contents of a footnote-reference branch.
///
/// A footnote-reference branch requires both the footnote's number and the
/// contents.
#[derive(Debug, PartialEq, Eq)]
pub struct FootnoteRef<'a> {
    pub number: u32,
    pub contents: &'a str,
}

impl FootnoteRef<'_> {
    /// Create a new [`FootnoteRef`] branch.
    fn new(number: u32, contents: &str) -> FootnoteRef {
        FootnoteRef { number, contents }
    }
}

/// Contents of a CrossRef branch.
///
/// Because a cross-reference branch consists of only the referred-footnote's
/// number, there is no need for a separate content field—the content is the
/// number.
#[derive(Debug, PartialEq, Eq)]
pub struct CrossRef {
    pub number: u32,
}

impl CrossRef {
    /// Create a new [`CrossRef`] branhc.
    fn new(number: u32) -> CrossRef {
        CrossRef { number }
    }
}

/// The complex type that the [`parser`] returns.
///
/// The parser returns two trees—one for each `.xml` file—and a vector
/// containing all of the footnotes that are referenced. This vector allows the
/// program to add bookmark markup only to those footnote references that need
/// it.
type ParseResults<'a> = (Vec<Branch<'a>>, Vec<Branch<'a>>, Vec<u32>);

/// The main parser function.
pub fn parser<'a>(
    doc_tokens: &'a [Token<'a>],
    fn_tokens: &'a [Token<'a>],
) -> Result<ParseResults<'a>, String> {
    debug!(slog_scope::logger(), "Starting parser...");

    let doc_branches =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "parse_fr()")), || {
            parse_fr(doc_tokens)
        }) {
            Ok(b) => b,
            Err(e) => return Err(e),
        };

    let (fn_branches, refd_fns) =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "parse_cr()")), || {
            parse_cr(fn_tokens)
        }) {
            Ok(b) => b,
            Err(e) => return Err(e),
        };

    debug!(slog_scope::logger(), "Parser finished.");
    Ok((doc_branches, fn_branches, refd_fns))
}

/// Parse the footnote references.
///
/// This function parses the tokens produced from the `document.xml` file.
/// Tokens with the [`TokenType`] `Other` are simply pushed as is. Tokens with
/// the [`TokenType`] `FootnoteRef` get a footnote number added, too.
///
/// Note, this function assumes that the starting footnote is 1. Use of Supra's
/// offset functionality will break this.
fn parse_fr<'a>(tokens: &'a [Token<'a>]) -> Result<Vec<Branch<'a>>, String> {
    debug!(slog_scope::logger(), "Starting document parser...");

    let mut parse: Vec<Branch> = Vec::new();
    let mut footnote_number = 1;

    for token in tokens {
        match token.token_type {
            TokenType::Other => {
                // Push the branch as is.
                trace!(
                    slog_scope::logger(),
                    "Pushing branch type Text containing {:?}",
                    token.contents
                );
                parse.push(Branch::Text(Text::new(token.contents)))
            }
            TokenType::FootnoteRef => {
                // Push the branch with a footnote number.
                trace!(
                    slog_scope::logger(),
                    "Pushing branch type FootnoteRef with footnote number {} and containing {}",
                    footnote_number,
                    token.contents
                );
                parse.push(Branch::FootnoteRef(FootnoteRef::new(
                    footnote_number,
                    token.contents,
                )));

                // Increment the footnote number for the next footnote.
                footnote_number += 1;
            }
            _ => {}
        }
    }

    debug!(slog_scope::logger(), "Document parser finished.");
    Ok(parse)
}

/// Parse the cross-reference.
///
/// This function parses the tokens produced from the `footnotes.xml` file.
/// Tokens with the [`TokenType`] `Other` are simply pushed as is. Tokens with
/// the [`TokenType`] `CrossRef` are parsed into a u32 number. This function
/// also returns a vector of all of the cross-referenced footnotes, which is
/// used to determine which footnote references in `document.xml` need bookmark
/// markup added.
fn parse_cr<'a>(tokens: &'a [Token<'a>]) -> Result<(Vec<Branch<'a>>, Vec<u32>), String> {
    debug!(slog_scope::logger(), "Starting footnotes parser...");

    let mut parse: Vec<Branch> = Vec::new();
    let mut referred_fns: Vec<u32> = Vec::new();

    for token in tokens {
        match token.token_type {
            TokenType::Other => {
                // Push the branch as is.
                trace!(
                    slog_scope::logger(),
                    "Pushing branch type Text containing {:?}",
                    token.contents
                );
                parse.push(Branch::Text(Text::new(token.contents)))
            }
            TokenType::CrossRef => {
                // Determine the number referred to.
                let footnote_number = match token.contents.parse::<u32>() {
                    Ok(n) => n,
                    Err(e) => {
                        let err_msg = format!("Error parsing cross references: {}", e);
                        return Err(err_msg);
                    }
                };

                // Determine if that footnote has been referenced before. If it
                // hasn't, add it to the list of referenced foototes.
                if !referred_fns.contains(&footnote_number) {
                    trace!(
                        slog_scope::logger(),
                        "Adding footnote {} to used cross-references",
                        footnote_number
                    );
                    referred_fns.push(footnote_number);
                }

                // Push the new branch.
                trace!(
                    slog_scope::logger(),
                    "Pushing branch type CrossRef for footnote {}",
                    footnote_number,
                );
                parse.push(Branch::CrossRef(CrossRef::new(footnote_number)))
            }
            _ => {}
        }
    }

    debug!(slog_scope::logger(), "Footnote parser finished.");
    Ok((parse, referred_fns))
}
