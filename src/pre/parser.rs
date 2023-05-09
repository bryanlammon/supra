//! This module contains the Supra parser. It converts collection of tokens into
//! the Supra syntax tree.

use crate::pre::lexer::{Token, TokenType};
use slog::{debug, o, trace};

/// The types of branches on the syntax tree.
#[derive(Debug, PartialEq)]
pub enum Branch<'a> {
    Text(Text<'a>),
    Footnote(Footnote<'a>),
    Citation(Citation<'a>),
    CrossRef(CrossRef<'a>),
    CiteBreak,
}

impl Branch<'_> {
    /// Used for debugging.
    #[allow(dead_code)]
    fn print_contents(&self) -> String {
        match self {
            Branch::Text(text) => text.contents.to_string(),
            Branch::Footnote(footnote) => {
                let mut raw = format!("{} {:?} ", footnote.number, footnote.id);
                for branch in &footnote.contents {
                    raw.push_str(&branch.print_contents());
                    raw.push(' ');
                }
                raw.trim().to_string()
            }
            Branch::Citation(citation) => format!(
                "{} {:?} {:?} {}",
                citation.reference, citation.pincite, citation.parenthetical, citation.punctuation
            ),
            Branch::CrossRef(crossref) => crossref.contents.to_string(),
            Branch::CiteBreak => "".to_string(),
        }
    }
}

/// Contents of a text branch.
#[derive(Debug, PartialEq, Eq)]
pub struct Text<'a> {
    pub contents: &'a str,
}

impl Text<'_> {
    fn new(contents: &str) -> Text {
        Text { contents }
    }
}

/// Contents of a footnote branch.
#[derive(Debug, PartialEq)]
pub struct Footnote<'a> {
    pub number: i32,
    pub id: Option<&'a str>,
    pub contents: Vec<Branch<'a>>,
}

impl Footnote<'_> {
    fn new<'b>(number: i32, id: Option<&'b str>, contents: Vec<Branch<'b>>) -> Footnote<'b> {
        Footnote {
            number,
            id,
            contents,
        }
    }
}

/// Contents of a citation branch.
#[derive(Debug, PartialEq, Eq)]
pub struct Citation<'a> {
    pub pre_cite: Option<PreCite<'a>>,
    pub reference: &'a str,
    pub pincite: Option<&'a str>,
    pub parenthetical: Option<&'a str>,
    pub punctuation: &'a str,
}

impl Citation<'_> {
    fn new<'b>(
        pre_cite: Option<PreCite<'b>>,
        reference: &'b str,
        pincite: Option<&'b str>,
        parenthetical: Option<&'b str>,
        punctuation: &'b str,
    ) -> Citation<'b> {
        Citation {
            pre_cite,
            reference,
            pincite,
            parenthetical,
            punctuation,
        }
    }
}

/// The pre-cite data.
#[derive(Debug, PartialEq, Eq)]
pub enum PreCite<'a> {
    Punctuation(Punctuation<'a>),
    Signal(Signal<'a>),
}

/// The pre-cite punctuation.
#[derive(Debug, PartialEq, Eq)]
pub struct Punctuation<'a> {
    pub contents: &'a str,
}

impl Punctuation<'_> {
    fn new(contents: &'_ str) -> Punctuation<'_> {
        Punctuation { contents }
    }
}

/// The pre-cite signal.
#[derive(Debug, PartialEq, Eq)]
pub struct Signal<'a> {
    pub contents: &'a str,
}

impl Signal<'_> {
    fn new(contents: &'_ str) -> Signal<'_> {
        Signal { contents }
    }
}

/// Contents of a crossref branch.
#[derive(Debug, PartialEq, Eq)]
pub struct CrossRef<'a> {
    pub contents: &'a str,
}

impl CrossRef<'_> {
    fn new(contents: &str) -> CrossRef {
        CrossRef { contents }
    }
}

/// A cite breaker.
#[derive(Debug, PartialEq, Eq)]
pub struct CiteBraker;

/// The main parser.
pub fn parser<'a>(tokens: &[Token<'a>], offset: i32) -> Result<Vec<Branch<'a>>, String> {
    debug!(slog_scope::logger(), "Starting parser...");

    match slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "text_parser()")),
        || text_parser(tokens, offset),
    ) {
        Ok(output) => {
            debug!(slog_scope::logger(), "Parsing completed.");
            Ok(output)
        }
        Err(e) => Err(e),
    }
}

/// Parse text and footnotes (sending footnotes to `footnote_parser`)
fn text_parser<'a>(tokens: &[Token<'a>], offset: i32) -> Result<Vec<Branch<'a>>, String> {
    trace!(slog_scope::logger(), "Starting text parser...");
    let mut parse: Vec<Branch> = Vec::new();
    let mut footnote_number = offset;
    let mut footnote_index: usize = 0;
    let mut open_footnote: bool = false;

    for (i, token) in tokens.iter().enumerate() {
        match token.token_type {
            TokenType::Text => {
                if !open_footnote {
                    trace!(
                        slog_scope::logger(),
                        "Pushing branch type Text containing {:?}",
                        token.contents
                    );
                    parse.push(Branch::Text(Text::new(token.contents)))
                };
            }
            TokenType::OpenFootnote => {
                footnote_number += 1;
                footnote_index = i;
                open_footnote = true;
            }
            TokenType::CloseFootnote => {
                open_footnote = false;
                let footnote_result = slog_scope::scope(
                    &slog_scope::logger().new(o!("fn" => "footnote_parser()")),
                    || footnote_parser(&tokens[footnote_index..i], footnote_number),
                );
                match footnote_result {
                    Ok(f) => {
                        trace!(
                            slog_scope::logger(),
                            "Pushing branch type Footnote containing {:?}",
                            f.contents
                        );
                        parse.push(Branch::Footnote(f));
                    }
                    Err(e) => return Err(e),
                }
            }
            _ => {}
        }
    }

    trace!(slog_scope::logger(), "Text parsing completed.");
    Ok(parse)
}

/// Parse the contents of a footnote into branches.
fn footnote_parser<'a>(tokens: &[Token<'a>], footnote_number: i32) -> Result<Footnote<'a>, String> {
    trace!(slog_scope::logger(), "Starting footnote parser...");
    let mut contents: Vec<Branch> = Vec::new();
    let mut id = None;
    let mut citation_index: usize = 0;
    let mut citation_started = false;

    for (i, token) in tokens.iter().enumerate() {
        match token.token_type {
            TokenType::ID => {
                trace!(slog_scope::logger(), "Adding id: {:?}", token.contents);
                id = Some(token.contents);
            }
            TokenType::PreCitePunctuation => {
                if !citation_started {
                    citation_started = true;
                    citation_index = i;
                }
            }
            TokenType::Signal => {
                if !citation_started {
                    citation_started = true;
                    citation_index = i;
                }
            }
            TokenType::Reference => {
                if !citation_started {
                    citation_started = true;
                    citation_index = i;
                };
            }
            TokenType::CitePunctuation => match slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "cite_parser()")),
                || cite_parser(&tokens[citation_index..i + 1]),
            ) {
                Ok(c) => {
                    trace!(
                        slog_scope::logger(),
                        "Pushing branch type Citation containing {:?}",
                        c
                    );
                    contents.push(Branch::Citation(c));
                    citation_started = false;
                }
                Err(e) => return Err(e),
            },
            TokenType::Text => {
                trace!(
                    slog_scope::logger(),
                    "Pushing branch type Text containing {:?}",
                    token.contents
                );
                contents.push(Branch::Text(Text::new(token.contents)));
            }
            TokenType::CrossRef => {
                trace!(
                    slog_scope::logger(),
                    "Pushing branch type CrossRef containing {:?}",
                    token.contents
                );
                contents.push(Branch::CrossRef(CrossRef::new(token.contents)));
            }
            TokenType::CiteBreak => {
                trace!(
                    slog_scope::logger(),
                    "Pushing branch type CiteBreak containing {:?}",
                    token.contents
                );
                contents.push(Branch::CiteBreak);
            }
            _ => {}
        }
    }

    trace!(slog_scope::logger(), "Footnote parsing completed.");
    Ok(Footnote::new(footnote_number, id, contents))
}

/// Parse the parts of a citation.
fn cite_parser<'a>(tokens: &[Token<'a>]) -> Result<Citation<'a>, String> {
    trace!(slog_scope::logger(), "Starting citation parser...");
    let mut citation = Citation::new(None, "", None, None, "");

    for token in tokens {
        match token.token_type {
            TokenType::PreCitePunctuation => {
                citation.pre_cite = Some(PreCite::Punctuation(Punctuation::new(token.contents)))
            }
            TokenType::Signal => {
                citation.pre_cite = Some(PreCite::Signal(Signal::new(token.contents)))
            }
            TokenType::Reference => citation.reference = token.contents,
            TokenType::Pincite => citation.pincite = pin_parser(token.contents),
            TokenType::Parenthetical => citation.parenthetical = Some(token.contents),
            TokenType::CitePunctuation => citation.punctuation = token.contents,
            _ => {}
        }
    }

    trace!(slog_scope::logger(), "Citation parsing completed.");
    Ok(citation)
}

/// Determine any pincites.
///
/// This function determines whether there's a pincite. If there is, it also
/// removes the "at."
fn pin_parser(input: &str) -> Option<&str> {
    if input.trim().is_empty() {
        None
    } else if !input.contains("at") {
        Some(input.trim())
    } else {
        let at_start = input.find("at").unwrap();
        Some(input[at_start + 2..].trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod text_parser {
        //
    }

    mod footnote_parser {
        use super::*;

        #[test]
        fn complex_footnote() {
            let footnote_tokens = vec![
                Token {
                    token_type: TokenType::ID,
                    contents: "[?complex_id]",
                },
                Token {
                    token_type: TokenType::Reference,
                    contents: "[@jones2021]",
                },
                Token {
                    token_type: TokenType::Pincite,
                    contents: " at 100",
                },
                Token {
                    token_type: TokenType::Parenthetical,
                    contents: "(explaining complicated stuff)",
                },
                Token {
                    token_type: TokenType::CitePunctuation,
                    contents: ".",
                },
                Token {
                    token_type: TokenType::Text,
                    contents: " This footnote is a bit complicated. *See, e.g.*, Source not in the library; ",
                },
                Token {
                    token_type: TokenType::CiteBreak,
                    contents: "[$]"
                },
                Token {
                    token_type: TokenType::Reference,
                    contents: "[@smith2020]",
                },
                Token {
                    token_type: TokenType::Pincite,
                    contents: " at 10--12",
                },
                Token {
                    token_type: TokenType::Parenthetical,
                    contents: "(super-complicated discussion)",
                },
                Token {
                    token_type: TokenType::CitePunctuation,
                    contents: ";",
                },
                Token {
                    token_type: TokenType::Reference,
                    contents: "[@johnson2019]",
                },
                Token {
                    token_type: TokenType::Pincite,
                    contents: " at 500, 550 n.5",
                },
                Token {
                    token_type: TokenType::Parenthetical,
                    contents: "(similarly complicated discussion)",
                },
                Token {
                    token_type: TokenType::CitePunctuation,
                    contents: ".",
                },
                Token {
                    token_type: TokenType::Text,
                    contents: " Here's some more text. Not to mention another citation. *Cf.* ",
                },
                Token {
                    token_type: TokenType::Reference,
                    contents: "[@Baker1900]",
                },
                Token {
                    token_type: TokenType::CitePunctuation,
                    contents: ".",
                },
                Token {
                    token_type: TokenType::Text,
                    contents: " For a further discussion, see *supra* notes ",
                },
                Token {
                    token_type: TokenType::CrossRef,
                    contents: "[?crossref1]",
                },
                Token {
                    token_type: TokenType::Text,
                    contents: "--",
                },
                Token {
                    token_type: TokenType::CrossRef,
                    contents: "[?crossref2]",
                },
                Token {
                    token_type: TokenType::Text,
                    contents: ".",
                },
            ];
            let footnote = footnote_parser(&footnote_tokens, 1).unwrap();

            assert_eq!(footnote.contents.len(), 12);
            assert_eq!(footnote.number, 1);
            assert_eq!(footnote.id, Some("[?complex_id]"));
            assert_eq!(
                footnote.contents[0].print_contents(),
                r#"[@jones2021] Some("100") Some("(explaining complicated stuff)") ."#
            );
            assert_eq!(
                footnote.contents[1].print_contents(),
                " This footnote is a bit complicated. *See, e.g.*, Source not in the library; "
            );
            assert_eq!(footnote.contents[2].print_contents(), r#""#);
            assert_eq!(
                footnote.contents[3].print_contents(),
                r#"[@smith2020] Some("10--12") Some("(super-complicated discussion)") ;"#
            );
            assert_eq!(
                footnote.contents[4].print_contents(),
                r#"[@johnson2019] Some("500, 550 n.5") Some("(similarly complicated discussion)") ."#
            );
            assert_eq!(
                footnote.contents[5].print_contents(),
                " Here's some more text. Not to mention another citation. *Cf.* "
            );
            assert_eq!(
                footnote.contents[6].print_contents(),
                "[@Baker1900] None None ."
            );
            assert_eq!(
                footnote.contents[7].print_contents(),
                " For a further discussion, see *supra* notes "
            );
            assert_eq!(footnote.contents[8].print_contents(), "[?crossref1]");
            assert_eq!(footnote.contents[9].print_contents(), "--");
            assert_eq!(footnote.contents[10].print_contents(), "[?crossref2]");
            assert_eq!(footnote.contents[11].print_contents(), ".");
        }
    }

    mod cite_parser {
        use super::*;

        #[test]
        fn basic_citation() {
            let input = vec![
                Token {
                    token_type: TokenType::Reference,
                    contents: "[@jones2021]",
                },
                Token {
                    token_type: TokenType::CitePunctuation,
                    contents: ".",
                },
            ];
            let citation = cite_parser(&input).unwrap();

            assert_eq!(citation.reference, "[@jones2021]");
            assert_eq!(citation.pincite, None);
            assert_eq!(citation.parenthetical, None);
            assert_eq!(citation.punctuation, ".");
        }

        #[test]
        fn complex_citation() {
            let input = vec![
                Token {
                    token_type: TokenType::Reference,
                    contents: "[@jones2021]",
                },
                Token {
                    token_type: TokenType::Pincite,
                    contents: r" at §\ 100.3[D](2)",
                },
                Token {
                    token_type: TokenType::Parenthetical,
                    contents: "(a parenthetical (with internal parentheses))",
                },
                Token::new(TokenType::CitePunctuation, "."),
            ];
            let citation = cite_parser(&input).unwrap();

            assert_eq!(citation.reference, "[@jones2021]");
            assert_eq!(citation.pincite, Some(r"§\ 100.3[D](2)"));
            assert_eq!(
                citation.parenthetical,
                Some("(a parenthetical (with internal parentheses))")
            );
            assert_eq!(citation.punctuation, ".");
        }
    }

    mod pin_parser {
        use super::*;

        #[test]
        fn pin_variety() {
            let pin0 = pin_parser(" ");
            let pin1 = pin_parser(r" at §\ 100.3[D](2)");
            let pin2 = pin_parser(r" at 100--01");
            let pin3 = pin_parser(r" at 100, 200");
            let pin4 = pin_parser(r" at 100 n.1");
            let pin5 = pin_parser(r" at 100 nn.1--45");
            let pin6 = pin_parser(r" at 500, 550 n.5");
            let pin7 = pin_parser("");

            assert_eq!(pin0, None);
            assert_eq!(pin7, None);
            assert_eq!(pin1, Some(r"§\ 100.3[D](2)"));
            assert_eq!(pin2, Some("100--01"));
            assert_eq!(pin3, Some("100, 200"));
            assert_eq!(pin4, Some("100 n.1"));
            assert_eq!(pin5, Some("100 nn.1--45"));
            assert_eq!(pin6, Some("500, 550 n.5"));
        }
    }
}
