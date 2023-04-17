//! This module contains the Supra lexer. It converts a string (formatted in
//! Pandoc-style markdown) into a collection of tokens.

use lazy_static::lazy_static;
use regex::Regex;
use slog::{debug, o, trace};

/// The lexer that works through the input string.
struct Lexer {
    start: usize,
    context: Context,
    last_char: Option<u8>,
    open_brackets: i32,
    open_parens: i32,
}

impl Lexer {
    /// Create a new lexer with essentially empty fields.
    fn new() -> Lexer {
        Lexer {
            start: 0,
            context: Context::None,
            last_char: None,
            open_brackets: 0,
            open_parens: 0,
        }
    }
}

/// The parts of a document that matter to Supra.
#[derive(Debug, PartialEq, Eq)]
enum Context {
    Text,
    Footnote,
    ID,
    Citation,
    CrossRef,
    Reference,
    Pincite,
    Parenthetical,
    CitePunctuation,
    CiteBreak,
    None,
}

/// The tokenized parts of the input string.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Token<'a> {
    pub token_type: TokenType,
    pub contents: &'a str,
}

impl Token<'_> {
    /// Creates a new [`Token`].
    pub fn new(token_type: TokenType, contents: &str) -> Token {
        Token {
            token_type,
            contents,
        }
    }
}

/// The types of tokens.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    Text,
    OpenFootnote,
    ID,
    Signal,
    PreCitePunctuation,
    Reference,
    Pincite,
    Parenthetical,
    CitePunctuation,
    CrossRef,
    CiteBreak,
    CloseFootnote,
}

/// The main lexer function.
pub fn lexer(input: &str) -> Result<Vec<Token>, String> {
    debug!(slog_scope::logger(), "Starting lexer...");

    match slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "text_lexer()")),
        || text_lexer(input),
    ) {
        Ok(output) => {
            debug!(slog_scope::logger(), "Lexing completed");
            Ok(output)
        }
        Err(e) => Err(e),
    }
}

/// The text lexer.
///
/// The text lexer is the first lexer function called. It should start with text
/// and then look for other types of tokens, sending them to other lexers when
/// needed.
///
/// It doesn't assume that the input starts with text. It instead accounts for
/// the possibility of starting with a footnote. Why? I have no idea.
fn text_lexer(input: &str) -> Result<Vec<Token>, String> {
    trace!(slog_scope::logger(), "Starting text lexer...");

    let mut lexer = Lexer::new();
    let mut lex: Vec<Token> = Vec::new();

    // Determine if we're starting with text or a footnote. I can't imagine
    // starting with a footnote, but I'm accounting for it.
    if input.as_bytes()[0] == b'^' && input.as_bytes()[1] == b'[' {
        lexer.context = Context::Footnote;
    } else {
        lexer.context = Context::Text;
    }

    for (i, c) in input.bytes().enumerate() {
        if lexer.context == Context::Text {
            if c == b'[' && lexer.last_char == Some(b'^') {
                // An opening footnote has been found. Crate a token from the
                // preceding text, move the lexer back a spot, and loop around
                // to work on the footnote.
                trace!(
                    slog_scope::logger(),
                    "Pushing token type {:?} containing {:?}",
                    TokenType::Text,
                    &input[lexer.start..i - 1]
                );
                lex.push(Token::new(TokenType::Text, &input[lexer.start..i - 1]));
                lexer.context = Context::Footnote;
                lexer.open_brackets = 0;
                lexer.start = i - 1;
            }
        } else if lexer.context == Context::Footnote {
            // This looks for the end of the footnote and then sends the entire
            // thing to the footnote lexer.
            if c == b'[' {
                // Found an open bracket to balance
                lexer.open_brackets += 1;
            } else if c == b']' && lexer.open_brackets > 0 {
                // Found a balancing closed bracket
                lexer.open_brackets -= 1;
            } else if c == b']' {
                // End of the footnote. Send the contents to the footnote
                // parser, which will return the contents as tokens.
                match slog_scope::scope(
                    &slog_scope::logger().new(o!("fn" => "footnote_lexer()")),
                    || footnote_lexer(&input[lexer.start..i + 1]),
                ) {
                    Ok(r) => {
                        // If the footnote lexer was successful, pus the opening
                        // footnote token, all tokens made by the footnote
                        // lexer, and the clossing footnote token.
                        trace!(
                            slog_scope::logger(),
                            "Pushing token type {:?} containing {:?}",
                            TokenType::OpenFootnote,
                            "^["
                        );
                        lex.push(Token::new(TokenType::OpenFootnote, "^["));

                        for token in r {
                            trace!(
                                slog_scope::logger(),
                                "Pushing token type {:?} containing {:?}",
                                token.token_type,
                                token.contents,
                            );
                            lex.push(token)
                        }

                        trace!(
                            slog_scope::logger(),
                            "Pushing token type {:?} containing {:?}",
                            TokenType::CloseFootnote,
                            "]"
                        );
                        lex.push(Token::new(TokenType::CloseFootnote, "]"));
                    }
                    Err(e) => return Err(e),
                };

                // Back to text, and move the lexer ahead one.
                lexer.context = Context::Text;
                lexer.start = i + 1;
            }
        }

        // End of the input...
        if i == input.bytes().len() - 1 && lexer.start != i {
            if lexer.context == Context::Text {
                trace!(
                    slog_scope::logger(),
                    "End of input; pushing token type {:?} containing {:?}",
                    TokenType::Text,
                    &input[lexer.start..i + 1]
                );
                lex.push(Token::new(TokenType::Text, &input[lexer.start..i + 1]));
            } else {
                return Err("The input string ends with an open footnote.\nEnsure footnotes contain matching brackets before running.".to_string());
            }
        }

        lexer.last_char = Some(c);
    }

    trace!(slog_scope::logger(), "Text lexer complete.");
    Ok(lex)
}

/// Lex the contents of a footnote.
///
/// Lexes the contents of a footnote (everything between the brackets) into
/// tokens. Footnotes can contain the following:
///
/// * A footnote ID.
/// * Text before and after a citation clause or sentence.
/// * The punctuation before a citation clause or sentence.
/// * Signals.
/// * The citation (see the citation lexer for its tokens).
/// * A cite breaker.
/// * A cross reference.
fn footnote_lexer(input: &str) -> Result<Vec<Token>, String> {
    trace!(
        slog_scope::logger(),
        "Starting footnote lexer for {}...",
        input
    );
    let mut lexer = Lexer::new();
    let mut lex: Vec<Token> = Vec::new();

    // Determine the starting block. It could be an ID, text (including pre-cite
    // text), a citation, or a cite braker.
    if input.as_bytes().get(2) == Some(&b'[') && input.as_bytes().get(3) == Some(&b'?') {
        // The start is an ID.
        lexer.context = Context::ID;
    } else if input.as_bytes().get(2) == Some(&b'[') && input.as_bytes().get(3) == Some(&b'@') {
        // The start is a citation.
        lexer.context = Context::Citation;
        // The number of open brackets needs to account for the opening footnote
        // bracket.
        lexer.open_brackets = -1;
    } else if input.as_bytes().get(2) == Some(&b'[') && input.as_bytes().get(3) == Some(&b'$') {
        // The start is a cite breaker.
        lexer.context = Context::CiteBreak;
    } else {
        lexer.context = Context::Text;
    }

    // The first two bytes are always b'^' and b'[', so skip those.
    lexer.start = 2;

    for (i, c) in input.bytes().enumerate() {
        if lexer.context == Context::ID && c == b']' {
            // Found the end of an ID
            trace!(
                slog_scope::logger(),
                "End of input; pushing token type {:?} containing {:?}",
                TokenType::ID,
                &input[lexer.start..i + 1]
            );
            lex.push(Token::new(TokenType::ID, &input[lexer.start..i + 1]));
            lexer.context = Context::Text;
            lexer.start = i + 1;
        } else if lexer.context == Context::CiteBreak && c == b']' {
            // Found the end of the cite break.
            trace!(
                slog_scope::logger(),
                "End of input; pushing token type {:?} containing {:?}",
                TokenType::CiteBreak,
                &input[lexer.start..i + 1]
            );
            lex.push(Token::new(TokenType::CiteBreak, &input[lexer.start..i + 1]));
            lexer.context = Context::Text;
            lexer.start = i + 1;
        } else if lexer.context == Context::Citation {
            // Look for the puncuation that ends the citation
            if (c == b' ' || c == b']')
                && (lexer.last_char == Some(b'.')
                    || lexer.last_char == Some(b',')
                    || lexer.last_char == Some(b';'))
                && lexer.open_brackets == 0
                && lexer.open_parens == 0
            {
                // End of the citation. Send the text to the citation parser.
                match slog_scope::scope(
                    &slog_scope::logger().new(o!("fn" => "cite_lexer()")),
                    || cite_lexer(&input[lexer.start..i]),
                ) {
                    Ok(r) => {
                        for token in r {
                            trace!(
                                slog_scope::logger(),
                                "Pushing token type {:?} containing {:?}",
                                token.token_type,
                                token.contents,
                            );
                            lex.push(token);
                        }
                    }
                    Err(e) => return Err(e),
                }
                lexer.context = Context::Text;
                lexer.start = i;
            } else if c == b'[' {
                lexer.open_brackets += 1;
            } else if c == b']' {
                lexer.open_brackets -= 1;
            } else if c == b'(' {
                lexer.open_parens += 1;
            } else if c == b')' {
                lexer.open_parens -= 1;
            }
        } else if lexer.context == Context::CrossRef && c == b']' {
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {:?}",
                TokenType::CrossRef,
                &input[lexer.start..i + 1],
            );

            lex.push(Token::new(TokenType::CrossRef, &input[lexer.start..i + 1]));

            lexer.context = Context::Text;
            lexer.start = i + 1;
        } else if lexer.context == Context::Text {
            // Look for a citation, crossref, or cite break.
            if c == b'@' && lexer.last_char == Some(b'[') {
                // If there was something before the citation, send it to the
                //pre-cite lexer. if !&input[lexer.start..i -
                //1].trim().is_empty() {

                //    trace!( slog_scope::logger(), "Pushing token type {:?}
                //        containing {:?}", TokenType::Text,
                //        &input[lexer.start..i - 1] );
                //        lex.push(Token::new(TokenType::Text,
                //        &input[lexer.start..i - 1]));
                //}

                if !&input[lexer.start..i - 1].is_empty() {
                    match slog_scope::scope(
                        &slog_scope::logger().new(o!("fn" => "pre_cite_lexer()")),
                        || pre_cite_lexer(&input[lexer.start..i - 1]),
                    ) {
                        Ok(r) => {
                            for token in r {
                                trace!(
                                    slog_scope::logger(),
                                    "Pushing token type {:?} containing {:?}",
                                    token.token_type,
                                    token.contents,
                                );
                                lex.push(token);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }

                lexer.context = Context::Citation;
                lexer.open_brackets = 1; // Expect a closing bracket for the cite
                lexer.open_parens = 0;
                lexer.start = i - 1;
            } else if c == b'?' && lexer.last_char == Some(b'[') {
                // If there was something before the cross ref, add it as a text
                // token.
                if !&input[lexer.start..i - 1].trim().is_empty() {
                    trace!(
                        slog_scope::logger(),
                        "Pushing token type {:?} containing {:?}",
                        TokenType::Text,
                        &input[lexer.start..i - 1]
                    );
                    lex.push(Token::new(TokenType::Text, &input[lexer.start..i - 1]));
                }

                lexer.context = Context::CrossRef;
                lexer.open_brackets = 1; // Expect a closing bracket for the crossref
                lexer.open_parens = 0;
                lexer.start = i - 1;
            } else if c == b'$' && lexer.last_char == Some(b'[') {
                // If there was something before the cite break, add it as a
                // text token.
                //if !&input[lexer.start..i - 1].is_empty() {
                //    trace!(
                //        slog_scope::logger(),
                //        "Pushing token type {:?} containing {:?}",
                //        TokenType::Text,
                //        &input[lexer.start..i - 1]
                //    );
                //    lex.push(Token::new(TokenType::Text, &input[lexer.start..i - 1]));

                //    lexer.context = Context::CiteBreak;
                //    lexer.open_brackets = 1; // Expect a closing bracket for the cite break
                //    lexer.open_parens = 0;
                //    lexer.start = i - 1;
                //}

                // If there was something before the cite break, send it to the pre-cite lexer.
                if !&input[lexer.start..i - 1].is_empty() {
                    match slog_scope::scope(
                        &slog_scope::logger().new(o!("fn" => "pre_cite_lexer()")),
                        || pre_cite_lexer(&input[lexer.start..i - 1]),
                    ) {
                        Ok(r) => {
                            for token in r {
                                trace!(
                                    slog_scope::logger(),
                                    "Pushing token type {:?} containing {:?}",
                                    token.token_type,
                                    token.contents,
                                );
                                lex.push(token);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }

                lexer.context = Context::CiteBreak;
                lexer.open_brackets = 1; // Expect a closing bracket for the cite
                lexer.open_parens = 0;
                lexer.start = i - 1;
            }
        }

        // End of the input...
        if i == input.bytes().len() - 1 {
            if lexer.context == Context::Text && c == b']' {
                // If there was something before the end, add it as a text
                // token.
                if !&input[lexer.start..i].trim().is_empty() {
                    trace!(
                        slog_scope::logger(),
                        "End of input; pushing token type {:?} containing {:?}",
                        TokenType::Text,
                        &input[lexer.start..i]
                    );
                    lex.push(Token::new(TokenType::Text, &input[lexer.start..i]));
                }
            } else {
                // I don't think this is possible (the call to the footnote
                // lexer requires a closing bracket), but I'm accounting for it.
                return Err(format!(
                    "The footnote \"{:?}\" does not end with a closing bracket.",
                    input
                ));
            }
        }

        lexer.last_char = Some(c);
    }

    trace!(slog_scope::logger(), "Footnote lexer complete.");
    Ok(lex)
}

/// Lex the text before a citation.
///
/// Lexes the text immediately preceding a citation sentence or clause. It looks
/// for a signal, the ending punctuation, and the text itself.
fn pre_cite_lexer(input: &str) -> Result<Vec<Token>, String> {
    // Regex for the pre-cite lexer.
    lazy_static! {
        /// Regex for finding the signal that ends pre-citation text.
        ///
        /// Each signal has three forms: (1) an italicized version that starts a
        /// sentence, (2) an italicized version that starts a clause, and (3) an
        /// un-italicized version that is used as part of a sentence.
        pub static ref SIGNAL: Regex = Regex::new("(?P<signal>\\*See generally, e\\.g\\.\\*,|\\*see generally, e\\.g\\.\\*,|see generally, e\\.g\\.,|\\*See generally\\*|\\*see generally\\*|see generally|\\*But cf\\., e\\.g\\.\\*,|\\*but cf\\., e\\.g\\.\\*,|but cf\\., e\\.g\\.,|\\*But cf\\.\\*|\\*but cf\\.\\*|but cf\\.|\\*But see, e\\.g\\.\\*,|\\*but see, e\\.g\\.\\*,|but see, e\\.g\\.,|\\*But see\\*|\\*but see\\*|but see|\\*Contra\\*|\\*contra\\*|contra|\\*Compare\\*|\\*compare\\*|compare|\\*with\\*|with|\\*Cf\\., e\\.g\\.\\*,|\\*cf\\., e\\.g\\.\\*,|cf\\., e\\.g\\.,|\\*Cf\\.\\*|\\*cf\\.\\*|cf\\.|\\*See also, e\\.g\\.\\*,|\\*see also, e\\.g\\.\\*,|see also, e\\.g\\.,|\\*See also\\*|\\*see also\\*|see also|\\*See, e\\.g\\.\\*,|\\*see, e\\.g\\.\\*,|see, e\\.g\\.,|\\*See\\*|\\*see\\*|see|\\*Accord\\*|\\*accord\\*|accord|\\*E\\.g\\.\\*|\\*e\\.g\\.\\*|e\\.g\\.)+\\s*$").unwrap();

        /// Regex for finding punctuation before a citation clause or sentence.
        ///
        /// This is used to determine whether to capitalize *Id.*s.
        ///
        /// Is this useful only if there is no signal??
        pub static ref PUNCTUATION: Regex = Regex::new("(?P<puncuation>\\?|!|:|;|,|\\.){1}\\s*$").unwrap();
    }

    trace!(
        slog_scope::logger(),
        "Starting pre-citation lexer for {}...",
        input
    );

    let mut lex: Vec<Token> = Vec::new();

    // Look for a signal.
    //
    // Because the regex looks for the signal followed by any whitespace and
    // then the end of the text, there should be zero or one matches in any
    // search.
    let signal = SIGNAL.find(input);

    //println!("{:?}", signal);

    //DEBUG//
    //if signal.is_some() {
    //    println!(
    //        "SIGNAL FOUND: {:?}",
    //        &input[signal.as_ref().unwrap().range()]
    //    );
    //} else {
    //    println!("NO SIGNAL FOUND");
    //}

    // Look for ending punctuation if there's no signal.
    let punctuation = match signal {
        Some(_) => None,
        None => PUNCTUATION.find(input),
    };

    //DEBUG
    // if punctuation.is_some() { println!( "PUNCTUATION FOUND: {:?}",
    //&input[punctuation.as_ref().unwrap().range()] ); } else { println!("NO
    //    PUNCTUATION FOUND") }

    // The rest is the text.
    if let Some(s) = signal {
        // If there's text before the signal, push it.
        if s.start() != 0 {
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {:?}",
                TokenType::Text,
                &input[0..s.start()]
            );
            //DEBUG
            // println!("PRE-SIGNAL TEXT: {}", &input[0..s.start()]);

            lex.push(Token::new(TokenType::Text, &input[0..s.start()]));
        }

        // Push the signal.
        trace!(
            slog_scope::logger(),
            "Pushing token type {:?} containing {:?}",
            TokenType::Signal,
            &input[s.range()]
        );
        //DEBUG//
        //println!("SIGNAL: {}", &input[s.range()]);

        lex.push(Token::new(TokenType::Signal, &input[s.range()]));
    } else if let Some(p) = punctuation {
        // If there's text before the punctuation (and there always should
        // be...), push it.
        if p.start() != 0 {
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {:?}",
                TokenType::Text,
                &input[0..p.start()]
            );
            //DEBUG// println!("PRE-PUNCTUATION TEXT: {}",
            //&input[0..p.start()]);

            lex.push(Token::new(TokenType::Text, &input[0..p.start()]));
        }
        // Push the punctuation.
        trace!(
            slog_scope::logger(),
            "Pushing token type {:?} containing {:?}",
            TokenType::PreCitePunctuation,
            &input[p.range()]
        );
        //DEBUG println!("PUNCTUATION: {}", &input[p.range()]);

        lex.push(Token::new(TokenType::PreCitePunctuation, &input[p.range()]));
    } else {
        trace!(
            slog_scope::logger(),
            "Pushing token type {:?} containing {:?}",
            TokenType::Text,
            &input
        );
        //DEBUG// println!("NO SIGNAL/PUNCTUATION; TEXT: {}", &input);

        lex.push(Token::new(TokenType::Text, input));
    }

    trace!(slog_scope::logger(), "Pre-citation lexer complete.");
    Ok(lex)
}

/// Lex a citation.
///
/// Lexes a citation, which requires a reference and ending punctuation, and
/// might include a pincite or parenthetical (or both).
fn cite_lexer(input: &str) -> Result<Vec<Token>, String> {
    trace!(
        slog_scope::logger(),
        "Starting citation lexer for {}...",
        input
    );

    let mut lexer = Lexer::new();
    lexer.context = Context::Reference;
    let mut lex: Vec<Token> = Vec::new();

    for (i, c) in input.bytes().enumerate() {
        if lexer.context == Context::Reference && c == b']' {
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {:?}",
                TokenType::Reference,
                &input[lexer.start..i + 1]
            );
            lex.push(Token::new(TokenType::Reference, &input[lexer.start..i + 1]));

            lexer.context = Context::Pincite;
            lexer.start = i + 1;
        } else if lexer.context == Context::Pincite && c == b'(' && lexer.last_char == Some(b' ') {
            if !&input[lexer.start..i - 1].trim().is_empty() {
                trace!(
                    slog_scope::logger(),
                    "Pushing token type {:?} containing {:?}",
                    TokenType::Pincite,
                    &input[lexer.start..i - 1]
                );
                lex.push(Token::new(TokenType::Pincite, &input[lexer.start..i - 1]))
            };

            lexer.context = Context::Parenthetical;
            lexer.start = i;
        } else if lexer.context == Context::Parenthetical {
            if c == b')' && lexer.open_parens == 0 {
                trace!(
                    slog_scope::logger(),
                    "Pushing token type {:?} containing {:?}",
                    TokenType::Parenthetical,
                    &input[lexer.start..i + 1]
                );
                lex.push(Token::new(
                    TokenType::Parenthetical,
                    &input[lexer.start..i + 1],
                ));

                lexer.context = Context::CitePunctuation;
                lexer.start = i + 1;
            } else if c == b'(' {
                lexer.open_parens += 1;
            } else if c == b')' {
                lexer.open_parens -= 1;
            }
        }

        // End of the input...
        if i == input.bytes().len() - 1 {
            if lexer.context == Context::Pincite && i > lexer.start {
                if !&input[lexer.start..i].trim().is_empty() {
                    trace!(
                        slog_scope::logger(),
                        "Pushing token type {:?} containing {:?}",
                        TokenType::Pincite,
                        &input[lexer.start..i]
                    );
                    lex.push(Token::new(TokenType::Pincite, &input[lexer.start..i]));
                };

                lexer.start = i;
            } else if lexer.context == Context::Parenthetical {
                return Err("No closing parentheses found for the parenthetical".to_string());
            }

            trace!(
                slog_scope::logger(),
                "End of input; pushing token type {:?} containing {:?}",
                TokenType::CitePunctuation,
                &input[lexer.start..input.bytes().len()]
            );
            lex.push(Token::new(
                TokenType::CitePunctuation,
                &input[lexer.start..input.bytes().len()],
            ));
        }

        lexer.last_char = Some(c);
    }

    trace!(slog_scope::logger(), "Citation lexer complete.");
    Ok(lex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_input() {
        let content = lexer(
            "This is a complex string.^[[?1] For more on complex strings, see [@jones2021] at 1 (explaining complex strings).]\nIt contains several lines,^[[?2] This is a midsentence footnote. But it doesn't have any citations.] some of which have footnotes,^[Like this one, which doesn't have an id. For a footnote with an Id, see *supra* note [?2].] some of which don't.\nThere are footnotes with lots of citations.^[[?lotsa_citations] Like this one, which cites to [@smith2021] at 100 (discussing stuff); [@jones2021] at 5 (discussing more stuff); and [@johnson2019a].]\nThere are also footnotes with no library citations.^[*See, e.g.*, [$] me.]\n\nAnd then there's some text in a new paragraph.",
        ).unwrap();

        assert_eq!(content.len(), 45);
        assert_eq!(content[0].contents, "This is a complex string.");
        assert_eq!(content[0].token_type, TokenType::Text);
        assert_eq!(content[1].contents, "^[");
        assert_eq!(content[1].token_type, TokenType::OpenFootnote);
        assert_eq!(content[2].contents, "[?1]");
        assert_eq!(content[2].token_type, TokenType::ID);
        assert_eq!(content[3].contents, " For more on complex strings, ");
        assert_eq!(content[3].token_type, TokenType::Text);
        assert_eq!(content[4].contents, "see ");
        assert_eq!(content[4].token_type, TokenType::Signal);
        assert_eq!(content[5].contents, "[@jones2021]");
        assert_eq!(content[5].token_type, TokenType::Reference);

        assert_eq!(content[6].contents, " at 1");
        assert_eq!(content[6].token_type, TokenType::Pincite);
        assert_eq!(content[7].contents, "(explaining complex strings)");
        assert_eq!(content[7].token_type, TokenType::Parenthetical);
        assert_eq!(content[8].contents, ".");
        assert_eq!(content[8].token_type, TokenType::CitePunctuation);
        assert_eq!(content[9].contents, "]");
        assert_eq!(content[9].token_type, TokenType::CloseFootnote);
        assert_eq!(content[10].contents, "\nIt contains several lines,");
        assert_eq!(content[10].token_type, TokenType::Text);

        assert_eq!(content[11].contents, "^[");
        assert_eq!(content[11].token_type, TokenType::OpenFootnote);
        assert_eq!(content[12].contents, "[?2]");
        assert_eq!(content[12].token_type, TokenType::ID);
        assert_eq!(
            content[13].contents,
            " This is a midsentence footnote. But it doesn't have any citations."
        );
        assert_eq!(content[13].token_type, TokenType::Text);
        assert_eq!(content[14].contents, "]");
        assert_eq!(content[14].token_type, TokenType::CloseFootnote);
        assert_eq!(content[15].contents, " some of which have footnotes,");
        assert_eq!(content[15].token_type, TokenType::Text);

        assert_eq!(content[16].contents, "^[");
        assert_eq!(content[16].token_type, TokenType::OpenFootnote);
        assert_eq!(
            content[17].contents,
            "Like this one, which doesn't have an id. For a footnote with an Id, see *supra* note "
        );
        assert_eq!(content[17].token_type, TokenType::Text);
        assert_eq!(content[18].contents, "[?2]");
        assert_eq!(content[18].token_type, TokenType::CrossRef);
        assert_eq!(content[19].contents, ".");
        assert_eq!(content[19].token_type, TokenType::Text);
        assert_eq!(content[20].contents, "]");
        assert_eq!(content[20].token_type, TokenType::CloseFootnote);

        assert_eq!(
            content[21].contents,
            " some of which don't.\nThere are footnotes with lots of citations."
        );
        assert_eq!(content[21].token_type, TokenType::Text);
        assert_eq!(content[22].contents, "^[");
        assert_eq!(content[22].token_type, TokenType::OpenFootnote);
        assert_eq!(content[23].contents, "[?lotsa_citations]");
        assert_eq!(content[23].token_type, TokenType::ID);
        assert_eq!(content[24].contents, " Like this one, which cites to ");
        assert_eq!(content[24].token_type, TokenType::Text);
        assert_eq!(content[25].contents, "[@smith2021]");
        assert_eq!(content[25].token_type, TokenType::Reference);

        assert_eq!(content[26].contents, " at 100");
        assert_eq!(content[26].token_type, TokenType::Pincite);
        assert_eq!(content[27].contents, "(discussing stuff)");
        assert_eq!(content[27].token_type, TokenType::Parenthetical);
        assert_eq!(content[28].contents, ";");
        assert_eq!(content[28].token_type, TokenType::CitePunctuation);
        assert_eq!(content[29].contents, " ");
        assert_eq!(content[29].token_type, TokenType::Text);
        assert_eq!(content[30].contents, "[@jones2021]");
        assert_eq!(content[30].token_type, TokenType::Reference);

        assert_eq!(content[31].contents, " at 5");
        assert_eq!(content[31].token_type, TokenType::Pincite);
        assert_eq!(content[32].contents, "(discussing more stuff)");
        assert_eq!(content[32].token_type, TokenType::Parenthetical);
        assert_eq!(content[33].contents, ";");
        assert_eq!(content[33].token_type, TokenType::CitePunctuation);
        assert_eq!(content[34].contents, " and ");
        assert_eq!(content[34].token_type, TokenType::Text);
        assert_eq!(content[35].contents, "[@johnson2019a]");
        assert_eq!(content[35].token_type, TokenType::Reference);

        assert_eq!(content[36].contents, ".");
        assert_eq!(content[36].token_type, TokenType::CitePunctuation);
        assert_eq!(content[37].contents, "]");
        assert_eq!(content[37].token_type, TokenType::CloseFootnote);
        assert_eq!(
            content[38].contents,
            "\nThere are also footnotes with no library citations."
        );
        assert_eq!(content[38].token_type, TokenType::Text);
        assert_eq!(content[39].contents, "^[");
        assert_eq!(content[39].token_type, TokenType::OpenFootnote);
        assert_eq!(content[40].contents, "*See, e.g.*, ");
        assert_eq!(content[40].token_type, TokenType::Signal);

        assert_eq!(content[41].contents, "[$]");
        assert_eq!(content[41].token_type, TokenType::CiteBreak);
        assert_eq!(content[42].contents, " me.");
        assert_eq!(content[42].token_type, TokenType::Text);
        assert_eq!(content[43].contents, "]");
        assert_eq!(content[43].token_type, TokenType::CloseFootnote);
        assert_eq!(
            content[44].contents,
            "\n\nAnd then there's some text in a new paragraph."
        );
        assert_eq!(content[44].token_type, TokenType::Text);
    }

    mod text_lexer {
        use super::*;

        #[test]
        fn one_line() {
            let content = text_lexer(r"This is the input string.").unwrap();
            assert_eq!(content.len(), 1);
            assert_eq!(content[0].contents, r"This is the input string.");
            assert_eq!(content[0].token_type, TokenType::Text)
        }

        #[test]
        fn one_footnote() {
            let content =
                text_lexer(r"This is the input string.^[This is the footnote.]\n").unwrap();
            assert_eq!(content.len(), 5);
            assert_eq!(content[0].contents, r"This is the input string.");
            assert_eq!(content[0].token_type, TokenType::Text);
            assert_eq!(content[1].contents, r"^[");
            assert_eq!(content[1].token_type, TokenType::OpenFootnote);
            assert_eq!(content[2].contents, r"This is the footnote.");
            assert_eq!(content[2].token_type, TokenType::Text);
            assert_eq!(content[3].contents, r"]");
            assert_eq!(content[3].token_type, TokenType::CloseFootnote);
            assert_eq!(content[4].contents, r"\n");
            assert_eq!(content[4].token_type, TokenType::Text);
        }

        #[test]
        fn text_footnote_text() {
            let content = text_lexer(r"This is the input string.^[This is the footnote.] This is the text that comes after the footnote.").unwrap();
            assert_eq!(content.len(), 5);
            assert_eq!(content[0].contents, r"This is the input string.");
            assert_eq!(content[0].token_type, TokenType::Text);
            assert_eq!(content[1].contents, r"^[");
            assert_eq!(content[1].token_type, TokenType::OpenFootnote);
            assert_eq!(content[2].contents, r"This is the footnote.");
            assert_eq!(content[2].token_type, TokenType::Text);
            assert_eq!(content[3].contents, r"]");
            assert_eq!(content[3].token_type, TokenType::CloseFootnote);
            assert_eq!(
                content[4].contents,
                r" This is the text that comes after the footnote."
            );
            assert_eq!(content[4].token_type, TokenType::Text);
        }

        #[test]
        fn multiple_footnotes_single_line() {
            let content = text_lexer(r"This is the input string,^[This is the first footnote.] with multiple footnotes on a single line.^[This is the second footnote.]").unwrap();
            assert_eq!(content.len(), 9);
            assert_eq!(content[0].contents, r"This is the input string,");
            assert_eq!(content[0].token_type, TokenType::Text);
            assert_eq!(content[1].contents, r"^[");
            assert_eq!(content[1].token_type, TokenType::OpenFootnote);
            assert_eq!(content[2].contents, r"This is the first footnote.");
            assert_eq!(content[2].token_type, TokenType::Text);
            assert_eq!(content[3].contents, r"]");
            assert_eq!(content[3].token_type, TokenType::CloseFootnote);
            assert_eq!(
                content[4].contents,
                r" with multiple footnotes on a single line."
            );
            assert_eq!(content[4].token_type, TokenType::Text);
            assert_eq!(content[5].contents, r"^[");
            assert_eq!(content[5].token_type, TokenType::OpenFootnote);
            assert_eq!(content[6].contents, r"This is the second footnote.");
            assert_eq!(content[6].token_type, TokenType::Text);
            assert_eq!(content[7].contents, r"]");
            assert_eq!(content[7].token_type, TokenType::CloseFootnote);
            assert_eq!(content[8].contents, r"");
            assert_eq!(content[8].token_type, TokenType::Text);
        }

        #[test]
        fn multiple_footnotes_multiple_lines() {
            let content = text_lexer("This is the input string.^[This is the first footnote.]\nThis is the text that comes after the first footnote.^[This is the second footnote.]\nThis is the final line of text.\n").unwrap();
            assert_eq!(content.len(), 9);
            assert_eq!(content[0].contents, r"This is the input string.");
            assert_eq!(content[0].token_type, TokenType::Text);
            assert_eq!(content[1].contents, r"^[");
            assert_eq!(content[1].token_type, TokenType::OpenFootnote);
            assert_eq!(content[2].contents, r"This is the first footnote.");
            assert_eq!(content[2].token_type, TokenType::Text);
            assert_eq!(content[3].contents, r"]");
            assert_eq!(content[3].token_type, TokenType::CloseFootnote);
            assert_eq!(
                content[4].contents,
                "\nThis is the text that comes after the first footnote."
            );
            assert_eq!(content[4].token_type, TokenType::Text);
            assert_eq!(content[5].contents, r"^[");
            assert_eq!(content[5].token_type, TokenType::OpenFootnote);
            assert_eq!(content[6].contents, r"This is the second footnote.");
            assert_eq!(content[6].token_type, TokenType::Text);
            assert_eq!(content[7].contents, r"]");
            assert_eq!(content[7].token_type, TokenType::CloseFootnote);
            assert_eq!(content[8].contents, "\nThis is the final line of text.\n");
            assert_eq!(content[8].token_type, TokenType::Text);
        }
    }

    mod footnote_lexer {
        use super::*;

        #[test]
        fn simple_footnote() {
            let footnote = footnote_lexer(r"^[This is a footnote.]").unwrap();
            assert_eq!(footnote.len(), 1);
            assert_eq!(footnote[0].contents, r"This is a footnote.");
            assert_eq!(footnote[0].token_type, TokenType::Text);
        }

        #[test]
        fn footnote_with_id() {
            let footnote = footnote_lexer(r"^[[?id] This is a footnote with an ID.]").unwrap();
            assert_eq!(footnote.len(), 2);
            assert_eq!(footnote[0].contents, r"[?id]");
            assert_eq!(footnote[0].token_type, TokenType::ID);
            assert_eq!(footnote[1].contents, r" This is a footnote with an ID.");
            assert_eq!(footnote[1].token_type, TokenType::Text);
        }

        #[test]
        fn footnote_with_citation() {
            let footnote =
                footnote_lexer(r"^[This is a footnote with a citation. *See* [@jones2021] at 100 (discussing stuff).]").unwrap();
            assert_eq!(footnote.len(), 6);
            assert_eq!(
                footnote[0].contents,
                r"This is a footnote with a citation. "
            );
            assert_eq!(footnote[0].token_type, TokenType::Text);
            assert_eq!(footnote[1].contents, r"*See* ");
            assert_eq!(footnote[1].token_type, TokenType::Signal);
            assert_eq!(footnote[2].contents, r"[@jones2021]");
            assert_eq!(footnote[2].token_type, TokenType::Reference);
            assert_eq!(footnote[3].contents, r" at 100");
            assert_eq!(footnote[3].token_type, TokenType::Pincite);
            assert_eq!(footnote[4].contents, r"(discussing stuff)");
            assert_eq!(footnote[4].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[5].contents, r".");
            assert_eq!(footnote[5].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn just_citation() {
            let footnote = footnote_lexer(r"^[[@jones2021].]").unwrap();
            assert_eq!(footnote.len(), 2);
            assert_eq!(footnote[0].contents, r"[@jones2021]");
            assert_eq!(footnote[0].token_type, TokenType::Reference);
            assert_eq!(footnote[1].contents, r".");
            assert_eq!(footnote[1].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn footnote_with_crossref() {
            let footnote =
                footnote_lexer(r"^[This footnote has a crossref. *See* *supra* note [?crossref].]")
                    .unwrap();
            assert_eq!(footnote.len(), 3);
            assert_eq!(
                footnote[0].contents,
                r"This footnote has a crossref. *See* *supra* note "
            );
            assert_eq!(footnote[0].token_type, TokenType::Text);
            assert_eq!(footnote[1].contents, r"[?crossref]");
            assert_eq!(footnote[1].token_type, TokenType::CrossRef);
            assert_eq!(footnote[2].contents, r".");
            assert_eq!(footnote[2].token_type, TokenType::Text);
        }

        #[test]
        fn complex_footnote() {
            let footnote = footnote_lexer(r"^[[?complex_id] [@jones2021] at 100 (explaining complicated stuff). This footnote is a bit complicated. *See, e.g.*, [@smith2020] at 10--12 (super-complicated discussion); [@johnson2019] at 500 (similarly complicated discussion). Here's some more text. Not to mention another citation. *Cf.* [@Baker1900]. For a further discussion, see *supra* notes [?crossref1]--[?crossref2].]").unwrap();

            //assert_eq!(footnote.len(), 26);
            assert_eq!(footnote[0].contents, r"[?complex_id]");
            assert_eq!(footnote[0].token_type, TokenType::ID);

            assert_eq!(footnote[1].contents, r" ");
            assert_eq!(footnote[1].token_type, TokenType::Text);

            assert_eq!(footnote[2].contents, r"[@jones2021]");
            assert_eq!(footnote[2].token_type, TokenType::Reference);
            assert_eq!(footnote[3].contents, r" at 100");
            assert_eq!(footnote[3].token_type, TokenType::Pincite);
            assert_eq!(footnote[4].contents, r"(explaining complicated stuff)");
            assert_eq!(footnote[4].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[5].contents, r".");
            assert_eq!(footnote[5].token_type, TokenType::CitePunctuation);
            assert_eq!(
                footnote[6].contents,
                r" This footnote is a bit complicated. "
            );
            assert_eq!(footnote[6].token_type, TokenType::Text);
            assert_eq!(footnote[7].contents, r"*See, e.g.*, ");
            assert_eq!(footnote[7].token_type, TokenType::Signal);
            assert_eq!(footnote[8].contents, r"[@smith2020]");
            assert_eq!(footnote[8].token_type, TokenType::Reference);
            assert_eq!(footnote[9].contents, r" at 10--12");
            assert_eq!(footnote[9].token_type, TokenType::Pincite);
            assert_eq!(footnote[10].contents, r"(super-complicated discussion)");
            assert_eq!(footnote[10].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[11].contents, r";");
            assert_eq!(footnote[11].token_type, TokenType::CitePunctuation);
            assert_eq!(footnote[12].contents, r" ");
            assert_eq!(footnote[12].token_type, TokenType::Text);
            assert_eq!(footnote[13].contents, r"[@johnson2019]");
            assert_eq!(footnote[13].token_type, TokenType::Reference);
            assert_eq!(footnote[14].contents, r" at 500");
            assert_eq!(footnote[14].token_type, TokenType::Pincite);
            assert_eq!(footnote[15].contents, r"(similarly complicated discussion)");
            assert_eq!(footnote[15].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[16].contents, r".");
            assert_eq!(footnote[16].token_type, TokenType::CitePunctuation);
            assert_eq!(
                footnote[17].contents,
                r" Here's some more text. Not to mention another citation. "
            );
            assert_eq!(footnote[17].token_type, TokenType::Text);
            assert_eq!(footnote[18].contents, r"*Cf.* ");
            assert_eq!(footnote[18].token_type, TokenType::Signal);
            assert_eq!(footnote[19].contents, r"[@Baker1900]");
            assert_eq!(footnote[19].token_type, TokenType::Reference);
            assert_eq!(footnote[20].contents, r".");
            assert_eq!(footnote[20].token_type, TokenType::CitePunctuation);
            assert_eq!(
                footnote[21].contents,
                r" For a further discussion, see *supra* notes "
            );
            assert_eq!(footnote[21].token_type, TokenType::Text);
            assert_eq!(footnote[22].contents, r"[?crossref1]");
            assert_eq!(footnote[22].token_type, TokenType::CrossRef);
            assert_eq!(footnote[23].contents, r"--");
            assert_eq!(footnote[23].token_type, TokenType::Text);
            assert_eq!(footnote[24].contents, r"[?crossref2]");
            assert_eq!(footnote[24].token_type, TokenType::CrossRef);
            assert_eq!(footnote[25].contents, r".");
            assert_eq!(footnote[25].token_type, TokenType::Text);
        }

        #[test]
        fn cite_breaker() {
            let footnote =
                footnote_lexer(r"^[This is a footnote with a cite breaker. *See* [$] Article Not in the Library at 1; [@jones2021] at 100 (discussing stuff).]").unwrap();
            assert_eq!(footnote.len(), 9);
            assert_eq!(
                footnote[0].contents,
                r"This is a footnote with a cite breaker. "
            );
            assert_eq!(footnote[0].token_type, TokenType::Text);
            assert_eq!(footnote[1].contents, "*See* ");
            assert_eq!(footnote[1].token_type, TokenType::Signal);
            assert_eq!(footnote[2].contents, r"[$]");
            assert_eq!(footnote[2].token_type, TokenType::CiteBreak);
            assert_eq!(footnote[3].contents, r" Article Not in the Library at 1");
            assert_eq!(footnote[3].token_type, TokenType::Text);
            assert_eq!(footnote[4].contents, r"; ");
            assert_eq!(footnote[4].token_type, TokenType::PreCitePunctuation);
            assert_eq!(footnote[5].contents, r"[@jones2021]");
            assert_eq!(footnote[5].token_type, TokenType::Reference);
            assert_eq!(footnote[6].contents, r" at 100");
            assert_eq!(footnote[6].token_type, TokenType::Pincite);
            assert_eq!(footnote[7].contents, r"(discussing stuff)");
            assert_eq!(footnote[7].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[8].contents, r".");
            assert_eq!(footnote[8].token_type, TokenType::CitePunctuation);
        }
    }

    mod pre_cite_lexer {
        use super::*;

        #[test]
        fn simple_pre_cite_signal() {
            let pre_cite = pre_cite_lexer(r"This is text before a signal. *See* ").unwrap();
            assert_eq!(pre_cite.len(), 2);
            assert_eq!(pre_cite[0].contents, r"This is text before a signal. ");
            assert_eq!(pre_cite[0].token_type, TokenType::Text);
            assert_eq!(pre_cite[1].contents, r"*See* ");
            assert_eq!(pre_cite[1].token_type, TokenType::Signal);
        }

        #[test]
        fn simple_pre_cite_punctuation() {
            let pre_cite = pre_cite_lexer(r"This is text before a citation. ").unwrap();
            assert_eq!(pre_cite.len(), 2);
            assert_eq!(pre_cite[0].contents, r"This is text before a citation");
            assert_eq!(pre_cite[0].token_type, TokenType::Text);
            assert_eq!(pre_cite[1].contents, r". ");
            assert_eq!(pre_cite[1].token_type, TokenType::PreCitePunctuation);
        }

        mod signal_tests {
            use super::*;

            #[test]
            fn test_eg() {
                // *E.g.*
                let s = pre_cite_lexer(r"This sentence the *E.g.* signal. *E.g.* ").unwrap();
                assert_eq!(s[1].contents, "*E.g.* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *e.g.*
                let s = pre_cite_lexer(r"This sentence the *e.g.* signal; *e.g.* ").unwrap();
                assert_eq!(s[1].contents, "*e.g.* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // e.g.
                let s = pre_cite_lexer(r"This sentence the e.g. signal, e.g. ").unwrap();
                assert_eq!(s[1].contents, "e.g. ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_accord() {
                // *Accord*
                let s = pre_cite_lexer(r"This sentence the *Accord* signal. *Accord* ").unwrap();
                assert_eq!(s[1].contents, "*Accord* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *accord*
                let s = pre_cite_lexer(r"This sentence the *accord* signal; *accord* ").unwrap();
                assert_eq!(s[1].contents, "*accord* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // accord
                let s = pre_cite_lexer(r"This sentence the accord signal; accord ").unwrap();
                assert_eq!(s[1].contents, "accord ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_see() {
                // *See*
                let s = pre_cite_lexer(r"This sentence the *See* signal. *See* ").unwrap();
                assert_eq!(s[1].contents, "*See* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *see*
                let s = pre_cite_lexer(r"This sentence the *see* signal; *see* ").unwrap();
                assert_eq!(s[1].contents, "*see* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // see
                let s = pre_cite_lexer(r"This sentence the see signal; see ").unwrap();
                assert_eq!(s[1].contents, "see ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_see_eg() {
                // *See, e.g.*,
                let s =
                    pre_cite_lexer(r"This sentence uses the `*See, e.g.*,` signal. *See, e.g.*, ")
                        .unwrap();
                assert_eq!(s[1].contents, "*See, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *see, e.g.*,
                let s =
                    pre_cite_lexer(r"This sentence uses the `*see, e.g.*,` signal; *see, e.g.*, ")
                        .unwrap();
                assert_eq!(s[1].contents, "*see, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // see, e.g.,
                let s = pre_cite_lexer(r"This sentence uses the `see, e.g.,` signal; see, e.g., ")
                    .unwrap();
                assert_eq!(s[1].contents, "see, e.g., ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_see_also() {
                // *See also*
                let s = pre_cite_lexer(r"This sentence uses the `*See also*` signal. *See also* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*See also* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *see also*
                let s = pre_cite_lexer(r"This sentence uses the `*see also*` signal; *see also* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*see also* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // see also
                let s =
                    pre_cite_lexer(r"This sentence uses the `see also` signal; see also ").unwrap();
                assert_eq!(s[1].contents, "see also ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_see_also_eg() {
                // *See also, e.g.*,
                let s = pre_cite_lexer(
                    r"This sentence uses the `*See also, e.g.*,` signal. *See also, e.g.*, ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*See also, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *see also, e.g.*,
                let s = pre_cite_lexer(
                    r"This sentence uses the `*see also, e.g.*,` signal; *see also, e.g.*, ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*see also, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // see also, e.g.
                let s = pre_cite_lexer(
                    r"This sentence uses the `see also, e.g.` signal; see also, e.g., ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "see also, e.g., ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_cf() {
                // *Cf.*
                let s = pre_cite_lexer(r"This sentence uses the `*Cf.*` signal. *Cf.* ").unwrap();
                assert_eq!(s[1].contents, "*Cf.* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *cf.*
                let s = pre_cite_lexer(r"This sentence uses the `*cf.*` signal. *cf.* ").unwrap();
                assert_eq!(s[1].contents, "*cf.* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // cf.
                let s = pre_cite_lexer(r"This sentence uses the `cf.` signal. cf. ").unwrap();
                assert_eq!(s[1].contents, "cf. ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_cf_eg() {
                // *Cf., e.g.*,
                let s =
                    pre_cite_lexer(r"This sentence uses the `*Cf., e.g.*,` signal. *Cf., e.g.*, ")
                        .unwrap();
                assert_eq!(s[1].contents, "*Cf., e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *cf., e.g.*,
                let s =
                    pre_cite_lexer(r"This sentence uses the `*cf., e.g.*,` signal. *cf., e.g.*, ")
                        .unwrap();
                assert_eq!(s[1].contents, "*cf., e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // cf., e.g.,
                let s = pre_cite_lexer(r"This sentence uses the `cf., e.g.,` signal. cf., e.g., ")
                    .unwrap();
                assert_eq!(s[1].contents, "cf., e.g., ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_compare() {
                // *Compare*
                let s = pre_cite_lexer(r"This sentence uses the `*Compare*` signal. *Compare* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*Compare* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *compare*
                let s = pre_cite_lexer(r"This sentence uses the `*compare*` signal. *compare* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*compare* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // compare
                let s =
                    pre_cite_lexer(r"This sentence uses the `compare` signal. compare ").unwrap();
                assert_eq!(s[1].contents, "compare ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_with() {
                // *with*
                let s = pre_cite_lexer(r"This sentence uses the `*with*` signal. *with* ").unwrap();
                assert_eq!(s[1].contents, "*with* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // with
                let s = pre_cite_lexer(r"This sentence uses the `with` signal. with ").unwrap();
                assert_eq!(s[1].contents, "with ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_contra() {
                // *Contra*
                let s =
                    pre_cite_lexer(r"This sentence uses the `*Contra*` signal. *Contra* ").unwrap();
                assert_eq!(s[1].contents, "*Contra* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *contra*
                let s =
                    pre_cite_lexer(r"This sentence uses the `*contra*` signal. *contra* ").unwrap();
                assert_eq!(s[1].contents, "*contra* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // contra
                let s = pre_cite_lexer(r"This sentence uses the `contra` signal. contra ").unwrap();
                assert_eq!(s[1].contents, "contra ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_but_see() {
                // *But see*
                let s = pre_cite_lexer(r"This sentence uses the `*But see*` signal. *But see* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*But see* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *but see*
                let s = pre_cite_lexer(r"This sentence uses the `*but see*` signal. *but see* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*but see* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // but see
                let s =
                    pre_cite_lexer(r"This sentence uses the `but see` signal. but see ").unwrap();
                assert_eq!(s[1].contents, "but see ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_but_see_eg() {
                // *But see, e.g.*,
                let s = pre_cite_lexer(
                    r"This sentence uses the `*But see, e.g.*,` signal. *But see, e.g.*, ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*But see, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *but see, e.g.*,
                let s = pre_cite_lexer(
                    r"This sentence uses the `*but see, e.g.*,` signal. *but see, e.g.*, ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*but see, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // but see, e.g.,
                let s = pre_cite_lexer(
                    r"This sentence uses the `but see, e.g.,` signal. but see, e.g., ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "but see, e.g., ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_but_cf() {
                // *But cf.*
                let s = pre_cite_lexer(r"This sentence uses the `*But cf.*` signal. *But cf.* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*But cf.* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *but cf.*
                let s = pre_cite_lexer(r"This sentence uses the `*but cf.*` signal. *but cf.* ")
                    .unwrap();
                assert_eq!(s[1].contents, "*but cf.* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // but cf.
                let s =
                    pre_cite_lexer(r"This sentence uses the `but cf.` signal. but cf. ").unwrap();
                assert_eq!(s[1].contents, "but cf. ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_but_cf_eg() {
                // *But cf., e.g.*,
                let s = pre_cite_lexer(
                    r"This sentence uses the `*But cf., e.g.*,` signal. *But cf., e.g.*, ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*But cf., e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *but cf., e.g.*,
                let s = pre_cite_lexer(
                    r"This sentence uses the `*but cf., e.g.*,` signal. *but cf., e.g.*, ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*but cf., e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // but cf., e.g.,
                let s = pre_cite_lexer(
                    r"This sentence uses the `but cf., e.g.,` signal. but cf., e.g., ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "but cf., e.g., ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_see_generally() {
                // *See generally*
                let s = pre_cite_lexer(
                    r"This sentence uses the `*See generally*` signal. *See generally* ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*See generally* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *see generally*
                let s = pre_cite_lexer(
                    r"This sentence uses the `*see generally*` signal. *see generally* ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "*see generally* ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // see generally
                let s = pre_cite_lexer(
                    r"This sentence uses the `see generally` signal. see generally ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "see generally ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }

            #[test]
            fn test_see_generally_eg() {
                // *See generally, e.g.*,
                let s = pre_cite_lexer(r"This sentence uses the `*See generally, e.g.*,` signal. *See generally, e.g.*, ").unwrap();
                assert_eq!(s[1].contents, "*See generally, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // *see generally, e.g.*,
                let s = pre_cite_lexer(r"This sentence uses the `*see generally, e.g.*,` signal. *see generally, e.g.*, ").unwrap();
                assert_eq!(s[1].contents, "*see generally, e.g.*, ");
                assert_eq!(s[1].token_type, TokenType::Signal);
                // see generally, e.g.,
                let s = pre_cite_lexer(
                    r"This sentence uses the `see generally, e.g.,` signal. see generally, e.g., ",
                )
                .unwrap();
                assert_eq!(s[1].contents, "see generally, e.g., ");
                assert_eq!(s[1].token_type, TokenType::Signal);
            }
        }

        mod punctuation_tests {
            use super::*;

            #[test]
            fn test_period() {
                let p = pre_cite_lexer("This sentence ends with a period. ").unwrap();
                assert_eq!(p[1].contents, ". ");
                assert_eq!(p[1].token_type, TokenType::PreCitePunctuation);
            }

            #[test]
            fn test_comma() {
                let p = pre_cite_lexer("This sentence ends with a comma, ").unwrap();
                assert_eq!(p[1].contents, ", ");
                assert_eq!(p[1].token_type, TokenType::PreCitePunctuation);
            }

            #[test]
            fn test_semicolon() {
                let p = pre_cite_lexer("This sentence ends with a semicolon; ").unwrap();
                assert_eq!(p[1].contents, "; ");
                assert_eq!(p[1].token_type, TokenType::PreCitePunctuation);
            }

            #[test]
            fn test_colon() {
                let p = pre_cite_lexer("This sentence ends with a colon: ").unwrap();
                assert_eq!(p[1].contents, ": ");
                assert_eq!(p[1].token_type, TokenType::PreCitePunctuation);
            }

            #[test]
            fn test_exclamation() {
                let p = pre_cite_lexer("This sentence ends with an exclamation point! ").unwrap();
                assert_eq!(p[1].contents, "! ");
                assert_eq!(p[1].token_type, TokenType::PreCitePunctuation);
            }

            #[test]
            fn test_question() {
                let p = pre_cite_lexer("This sentence ends with a question mark? ").unwrap();
                assert_eq!(p[1].contents, "? ");
                assert_eq!(p[1].token_type, TokenType::PreCitePunctuation);
            }
        }

        #[test]
        fn no_text() {
            let pre_cite = pre_cite_lexer(r"*See* ").unwrap();
            assert_eq!(pre_cite.len(), 1);
            assert_eq!(pre_cite[0].contents, "*See* ");
            assert_eq!(pre_cite[0].token_type, TokenType::Signal);
        }
    }

    mod cite_lexer {
        use super::*;

        #[test]
        fn simple_citation() {
            let citation = cite_lexer(r"[@jones2021].").unwrap();
            assert_eq!(citation.len(), 2);
            assert_eq!(citation[0].contents, r"[@jones2021]");
            assert_eq!(citation[0].token_type, TokenType::Reference);
            assert_eq!(citation[1].contents, r".");
            assert_eq!(citation[1].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn punctuation_comma() {
            let citation = cite_lexer(r"[@jones2021],").unwrap();
            assert_eq!(citation.len(), 2);
            assert_eq!(citation[0].contents, r"[@jones2021]");
            assert_eq!(citation[0].token_type, TokenType::Reference);
            assert_eq!(citation[1].contents, r",");
            assert_eq!(citation[1].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn pincite() {
            let citation = cite_lexer(r"[@jones2021] at 100;").unwrap();
            assert_eq!(citation.len(), 3);
            assert_eq!(citation[0].contents, r"[@jones2021]");
            assert_eq!(citation[0].token_type, TokenType::Reference);
            assert_eq!(citation[1].contents, r" at 100");
            assert_eq!(citation[1].token_type, TokenType::Pincite);
            assert_eq!(citation[2].contents, r";");
            assert_eq!(citation[2].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn parenthetical() {
            let citation = cite_lexer(r"[@jones2021] (a parenthetical);").unwrap();
            assert_eq!(citation.len(), 3);
            assert_eq!(citation[0].contents, r"[@jones2021]");
            assert_eq!(citation[0].token_type, TokenType::Reference);
            assert_eq!(citation[1].contents, r"(a parenthetical)");
            assert_eq!(citation[1].token_type, TokenType::Parenthetical);
            assert_eq!(citation[2].contents, r";");
            assert_eq!(citation[2].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn balanced_parentheses() {
            let citation =
                cite_lexer(r"[@jones2021] (a parenthetical (with internal parentheses));").unwrap();
            assert_eq!(citation.len(), 3);
            assert_eq!(citation[0].contents, r"[@jones2021]");
            assert_eq!(citation[0].token_type, TokenType::Reference);
            assert_eq!(
                citation[1].contents,
                r"(a parenthetical (with internal parentheses))"
            );
            assert_eq!(citation[1].token_type, TokenType::Parenthetical);
            assert_eq!(citation[2].contents, r";");
            assert_eq!(citation[2].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn unbalanced_parentheses() {
            let citation =
                cite_lexer(r"[@jones2021] (a parenthetical (with unbalanced parentheses);");
            assert!(citation.is_err());
        }

        #[test]
        fn pins_and_parentheticals() {
            let citation =
                cite_lexer(r"[@jones2021] at 100 (a parenthetical (with internal parentheses));")
                    .unwrap();
            assert_eq!(citation.len(), 4);
            assert_eq!(citation[0].contents, r"[@jones2021]");
            assert_eq!(citation[0].token_type, TokenType::Reference);
            assert_eq!(citation[1].contents, r" at 100");
            assert_eq!(citation[1].token_type, TokenType::Pincite);
            assert_eq!(
                citation[2].contents,
                r"(a parenthetical (with internal parentheses))"
            );
            assert_eq!(citation[2].token_type, TokenType::Parenthetical);
            assert_eq!(citation[3].contents, r";");
            assert_eq!(citation[3].token_type, TokenType::CitePunctuation);
        }

        #[test]
        fn pin_variety() {
            let citation1 = cite_lexer(r"[@jones2021] at §\ 100.3[D](2);").unwrap();
            let citation2 = cite_lexer(r"[@jones2021] at 100--01;").unwrap();
            let citation3 = cite_lexer(r"[@jones2021] at 100, 200;").unwrap();
            let citation4 = cite_lexer(r"[@jones2021] at 100 n.1;").unwrap();
            let citation5 = cite_lexer(r"[@jones2021] at 100 nn.1--45;").unwrap();
            let citation6 =
                cite_lexer(r"[@johnson2019] at 500, 550 n.5 (parenthetical after the pincite).")
                    .unwrap();
            let citation7 = cite_lexer(r"[@jones2021] 100--01;").unwrap();

            assert_eq!(citation1[1].contents, r" at §\ 100.3[D](2)");
            assert_eq!(citation1[1].token_type, TokenType::Pincite);
            assert_eq!(citation2[1].contents, r" at 100--01");
            assert_eq!(citation2[1].token_type, TokenType::Pincite);
            assert_eq!(citation3[1].contents, r" at 100, 200");
            assert_eq!(citation3[1].token_type, TokenType::Pincite);
            assert_eq!(citation4[1].contents, r" at 100 n.1");
            assert_eq!(citation4[1].token_type, TokenType::Pincite);
            assert_eq!(citation5[1].contents, r" at 100 nn.1--45");
            assert_eq!(citation5[1].token_type, TokenType::Pincite);
            assert_eq!(citation6[1].contents, r" at 500, 550 n.5");
            assert_eq!(citation6[1].token_type, TokenType::Pincite);
            assert_eq!(citation7[1].contents, r" 100--01");
            assert_eq!(citation7[1].token_type, TokenType::Pincite);
        }
    }
}
