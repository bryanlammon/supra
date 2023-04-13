//! This module contains the Supra lexer. It converts a string (formatted in
//! Pandoc-style markdown) into a collection of tokens.

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
    Punctuation,
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
/// tokens.
fn footnote_lexer(input: &str) -> Result<Vec<Token>, String> {
    trace!(
        slog_scope::logger(),
        "Starting footnote lexer for {}...",
        input
    );
    let mut lexer = Lexer::new();
    let mut lex: Vec<Token> = Vec::new();

    // Determine the starting block. It could be an ID, a citation, or text.
    if input.as_bytes().get(2) == Some(&b'[') && input.as_bytes().get(3) == Some(&b'?') {
        lexer.context = Context::ID;
    } else if input.as_bytes().get(2) == Some(&b'[') && input.as_bytes().get(3) == Some(&b'@') {
        lexer.context = Context::Citation;
        // The number of open brackets needs to account for the opening footnote
        // bracket.
        lexer.open_brackets = -1;
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
                && (lexer.last_char == Some(b';') || lexer.last_char == Some(b'.'))
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
                // If there was something before the citation, add it as a text
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

                    lexer.context = Context::CiteBreak;
                    lexer.open_brackets = 1; // Expect a closing bracket for the cite break
                    lexer.open_parens = 0;
                    lexer.start = i - 1;
                }
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

                lexer.context = Context::Punctuation;
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
            "This is a complex string.^[[?1] For more on complex strings, see [@jones2021] at 1 (explaining complex strings).]\nIt contains several lines,^[[?2] This is a midsentence footnote. But it doesn't have any citations.] some of which have footnotes,^[Like this one, which doesn't have an id. For a footnote with an Id, see *supra* note [?2].] some of which don't.\nThere are footnotes with lots of citations.^[[?lotsa_citations] Like this one, which cites to [@smith2021] at 100 (discussing stuff); [@jones2021] at 5 (discussing more stuff); and [@johnson2019a].]\nThere are also footnotes with no citations.^[*See, e.g.*, me.]\n\nAnd then there's some text in a new paragraph.",
        ).unwrap();

        assert_eq!(content.len(), 41);
        assert_eq!(content[0].contents, "This is a complex string.");
        assert_eq!(content[0].token_type, TokenType::Text);
        assert_eq!(content[1].contents, "^[");
        assert_eq!(content[1].token_type, TokenType::OpenFootnote);
        assert_eq!(content[2].contents, "[?1]");
        assert_eq!(content[2].token_type, TokenType::ID);
        assert_eq!(content[3].contents, " For more on complex strings, see ");
        assert_eq!(content[3].token_type, TokenType::Text);
        assert_eq!(content[4].contents, "[@jones2021]");
        assert_eq!(content[4].token_type, TokenType::Reference);

        assert_eq!(content[5].contents, " at 1");
        assert_eq!(content[5].token_type, TokenType::Pincite);
        assert_eq!(content[6].contents, "(explaining complex strings)");
        assert_eq!(content[6].token_type, TokenType::Parenthetical);
        assert_eq!(content[7].contents, ".");
        assert_eq!(content[7].token_type, TokenType::CitePunctuation);
        assert_eq!(content[8].contents, "]");
        assert_eq!(content[8].token_type, TokenType::CloseFootnote);
        assert_eq!(content[9].contents, "\nIt contains several lines,");
        assert_eq!(content[9].token_type, TokenType::Text);

        assert_eq!(content[10].contents, "^[");
        assert_eq!(content[10].token_type, TokenType::OpenFootnote);
        assert_eq!(content[11].contents, "[?2]");
        assert_eq!(content[11].token_type, TokenType::ID);
        assert_eq!(
            content[12].contents,
            " This is a midsentence footnote. But it doesn't have any citations."
        );
        assert_eq!(content[12].token_type, TokenType::Text);
        assert_eq!(content[13].contents, "]");
        assert_eq!(content[13].token_type, TokenType::CloseFootnote);
        assert_eq!(content[14].contents, " some of which have footnotes,");
        assert_eq!(content[14].token_type, TokenType::Text);

        assert_eq!(content[15].contents, "^[");
        assert_eq!(content[15].token_type, TokenType::OpenFootnote);
        assert_eq!(
            content[16].contents,
            "Like this one, which doesn't have an id. For a footnote with an Id, see *supra* note "
        );
        assert_eq!(content[16].token_type, TokenType::Text);
        assert_eq!(content[17].contents, "[?2]");
        assert_eq!(content[17].token_type, TokenType::CrossRef);
        assert_eq!(content[18].contents, ".");
        assert_eq!(content[18].token_type, TokenType::Text);
        assert_eq!(content[19].contents, "]");
        assert_eq!(content[19].token_type, TokenType::CloseFootnote);

        assert_eq!(
            content[20].contents,
            " some of which don't.\nThere are footnotes with lots of citations."
        );
        assert_eq!(content[20].token_type, TokenType::Text);
        assert_eq!(content[21].contents, "^[");
        assert_eq!(content[21].token_type, TokenType::OpenFootnote);
        assert_eq!(content[22].contents, "[?lotsa_citations]");
        assert_eq!(content[22].token_type, TokenType::ID);
        assert_eq!(content[23].contents, " Like this one, which cites to ");
        assert_eq!(content[23].token_type, TokenType::Text);
        assert_eq!(content[24].contents, "[@smith2021]");
        assert_eq!(content[24].token_type, TokenType::Reference);

        assert_eq!(content[25].contents, " at 100");
        assert_eq!(content[25].token_type, TokenType::Pincite);
        assert_eq!(content[26].contents, "(discussing stuff)");
        assert_eq!(content[26].token_type, TokenType::Parenthetical);
        assert_eq!(content[27].contents, ";");
        assert_eq!(content[27].token_type, TokenType::CitePunctuation);
        assert_eq!(content[28].contents, "[@jones2021]");
        assert_eq!(content[28].token_type, TokenType::Reference);
        assert_eq!(content[29].contents, " at 5");
        assert_eq!(content[29].token_type, TokenType::Pincite);

        assert_eq!(content[30].contents, "(discussing more stuff)");
        assert_eq!(content[30].token_type, TokenType::Parenthetical);
        assert_eq!(content[31].contents, ";");
        assert_eq!(content[31].token_type, TokenType::CitePunctuation);
        assert_eq!(content[32].contents, " and ");
        assert_eq!(content[32].token_type, TokenType::Text);
        assert_eq!(content[33].contents, "[@johnson2019a]");
        assert_eq!(content[33].token_type, TokenType::Reference);
        assert_eq!(content[34].contents, ".");
        assert_eq!(content[34].token_type, TokenType::CitePunctuation);

        assert_eq!(content[35].contents, "]");
        assert_eq!(content[35].token_type, TokenType::CloseFootnote);
        assert_eq!(
            content[36].contents,
            "\nThere are also footnotes with no citations."
        );
        assert_eq!(content[36].token_type, TokenType::Text);
        assert_eq!(content[37].contents, "^[");
        assert_eq!(content[37].token_type, TokenType::OpenFootnote);
        assert_eq!(content[38].contents, "*See, e.g.*, me.");
        assert_eq!(content[38].token_type, TokenType::Text);
        assert_eq!(content[39].contents, "]");
        assert_eq!(content[39].token_type, TokenType::CloseFootnote);

        assert_eq!(
            content[40].contents,
            "\n\nAnd then there's some text in a new paragraph."
        );
        assert_eq!(content[40].token_type, TokenType::Text);
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
            assert_eq!(footnote.len(), 5);
            assert_eq!(
                footnote[0].contents,
                r"This is a footnote with a citation. *See* "
            );
            assert_eq!(footnote[0].token_type, TokenType::Text);
            assert_eq!(footnote[1].contents, r"[@jones2021]");
            assert_eq!(footnote[1].token_type, TokenType::Reference);
            assert_eq!(footnote[2].contents, r" at 100");
            assert_eq!(footnote[2].token_type, TokenType::Pincite);
            assert_eq!(footnote[3].contents, r"(discussing stuff)");
            assert_eq!(footnote[3].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[4].contents, r".");
            assert_eq!(footnote[4].token_type, TokenType::CitePunctuation);
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
            let footnote = footnote_lexer(r"^[[?complex_id] [@jones2021] at 100 (explaining complicated stuff). This footnote is a bit complicated. *See, e.g.*, [@smith2020] at 10--12 (super-complicated discussion); [@johnson2019] at 500, 550 n.5 (similarly complicated discussion). Here's some more text. Not to mention another citation. *Cf.* [@Baker1900]. For a further discussion, see *supra* notes [?crossref1]--[?crossref2].]").unwrap();

            assert_eq!(footnote.len(), 22);
            assert_eq!(footnote[0].contents, r"[?complex_id]");
            assert_eq!(footnote[0].token_type, TokenType::ID);
            assert_eq!(footnote[1].contents, r"[@jones2021]");
            assert_eq!(footnote[1].token_type, TokenType::Reference);
            assert_eq!(footnote[2].contents, r" at 100");
            assert_eq!(footnote[2].token_type, TokenType::Pincite);
            assert_eq!(footnote[3].contents, r"(explaining complicated stuff)");
            assert_eq!(footnote[3].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[4].contents, r".");
            assert_eq!(footnote[4].token_type, TokenType::CitePunctuation);
            assert_eq!(
                footnote[5].contents,
                r" This footnote is a bit complicated. *See, e.g.*, "
            );
            assert_eq!(footnote[5].token_type, TokenType::Text);
            assert_eq!(footnote[6].contents, r"[@smith2020]");
            assert_eq!(footnote[6].token_type, TokenType::Reference);
            assert_eq!(footnote[7].contents, r" at 10--12");
            assert_eq!(footnote[7].token_type, TokenType::Pincite);
            assert_eq!(footnote[8].contents, r"(super-complicated discussion)");
            assert_eq!(footnote[8].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[9].contents, r";");
            assert_eq!(footnote[9].token_type, TokenType::CitePunctuation);
            assert_eq!(footnote[10].contents, r"[@johnson2019]");
            assert_eq!(footnote[10].token_type, TokenType::Reference);
            assert_eq!(footnote[11].contents, r" at 500, 550 n.5");
            assert_eq!(footnote[11].token_type, TokenType::Pincite);
            assert_eq!(footnote[12].contents, r"(similarly complicated discussion)");
            assert_eq!(footnote[12].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[13].contents, r".");
            assert_eq!(footnote[13].token_type, TokenType::CitePunctuation);
            assert_eq!(
                footnote[14].contents,
                r" Here's some more text. Not to mention another citation. *Cf.* "
            );
            assert_eq!(footnote[14].token_type, TokenType::Text);
            assert_eq!(footnote[15].contents, r"[@Baker1900]");
            assert_eq!(footnote[15].token_type, TokenType::Reference);
            assert_eq!(footnote[16].contents, r".");
            assert_eq!(footnote[16].token_type, TokenType::CitePunctuation);
            assert_eq!(
                footnote[17].contents,
                r" For a further discussion, see *supra* notes "
            );
            assert_eq!(footnote[17].token_type, TokenType::Text);
            assert_eq!(footnote[18].contents, r"[?crossref1]");
            assert_eq!(footnote[18].token_type, TokenType::CrossRef);
            assert_eq!(footnote[19].contents, r"--");
            assert_eq!(footnote[19].token_type, TokenType::Text);
            assert_eq!(footnote[20].contents, r"[?crossref2]");
            assert_eq!(footnote[20].token_type, TokenType::CrossRef);
            assert_eq!(footnote[21].contents, r".");
            assert_eq!(footnote[21].token_type, TokenType::Text);
        }

        #[test]
        fn cite_breaker() {
            let footnote =
                footnote_lexer(r"^[This is a footnote with a cite breaker. *See* [$] Article Not in the Library at 1; [@jones2021] at 100 (discussing stuff).]").unwrap();
            assert_eq!(footnote.len(), 7);
            assert_eq!(
                footnote[0].contents,
                r"This is a footnote with a cite breaker. *See* "
            );
            assert_eq!(footnote[0].token_type, TokenType::Text);
            assert_eq!(footnote[1].contents, r"[$]");
            assert_eq!(footnote[1].token_type, TokenType::CiteBreak);
            assert_eq!(footnote[2].contents, r" Article Not in the Library at 1; ");
            assert_eq!(footnote[2].token_type, TokenType::Text);
            assert_eq!(footnote[3].contents, r"[@jones2021]");
            assert_eq!(footnote[3].token_type, TokenType::Reference);
            assert_eq!(footnote[4].contents, r" at 100");
            assert_eq!(footnote[4].token_type, TokenType::Pincite);
            assert_eq!(footnote[5].contents, r"(discussing stuff)");
            assert_eq!(footnote[5].token_type, TokenType::Parenthetical);
            assert_eq!(footnote[6].contents, r".");
            assert_eq!(footnote[6].token_type, TokenType::CitePunctuation);
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
