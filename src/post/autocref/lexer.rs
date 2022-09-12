//! This module contains the lexer for breaking down the xml files. It
//! identifies the chunks of potential bookmarks (for `document.xml`) and
//! cross-references (for `footnotes.xml) as well as chunks containing
//! everything else.

use regex::Regex;
use slog::{debug, o, trace};

/// The lexer that works through an input string.
///
/// This keeps track of the starting index for each chunk.
struct Lexer {
    start: usize,
}

impl Lexer {
    /// Create a new lexer that starts at index 0.
    fn new() -> Lexer {
        Lexer { start: 0 }
    }
}

/// The tokens of the input string.
///
/// Tokens consist of a [`TokenType`] and contents. The contents refer to a
/// slice of the input string.
#[derive(Debug, PartialEq, Eq)]
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

/// The types of tokens in the documents.
///
/// A `FootnoteRef` refers to a chunk containing the markup for a footnote
/// reference in `document.xml`.
///
/// A `CrossRef` refers to a chunk containing the number referencing another
/// footnote in `footnotes.xml`.
///
/// Everything else is `Other`.
#[derive(Debug, PartialEq, Eq)]
pub enum TokenType {
    CrossRef,
    FootnoteRef,
    Other,
}

/// The main lexer function.
///
/// This is a parent function for the two separate lexers.
pub fn lex<'a>(
    doc_input: &'a str,
    fn_input: &'a str,
) -> Result<(Vec<Token<'a>>, Vec<Token<'a>>), String> {
    debug!(slog_scope::logger(), "Starting lexer...");

    // First get the tokens from doc_input
    let doc_lex =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "lex_doc()")), || {
            lex_doc(doc_input)
        }) {
            Ok(l) => l,
            Err(e) => return Err(e),
        };

    // Then get the tokens from fn_input
    let fn_lex = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "lex_fn()")), || {
        lex_fn(fn_input)
    }) {
        Ok(l) => l,
        Err(e) => return Err(e),
    };

    debug!(slog_scope::logger(), "Lexer finished.");
    Ok((doc_lex, fn_lex))
}

/// Lex the contents of document.xml.
///
/// This function uses regex to identify the footnote references in
/// `document.xml`. It then uses the index of those points to create tokens of
/// the [`TokenType`] `FootnoteRef` or `Other`.
fn lex_doc(doc_input: &str) -> Result<Vec<Token>, String> {
    debug!(slog_scope::logger(), "Lexing document...");

    // Create a new lexer and empty vector of tokens
    let mut lexer = Lexer::new();
    let mut lex: Vec<Token> = Vec::new();

    // Use regex to identify each match
    let re = Regex::new(
        r#"(<w:r><w:rPr><w:rStyle w:val="FootnoteReference" /></w:rPr><w:footnoteReference w:id=")([0-9]{1,9})(" /></w:r>)"#
    ).unwrap();
    for mat in re.find_iter(doc_input) {
        // The file should always start with an other chunk. And this loop
        // always ends with a new other chunk. So each loop should start by
        // closing off an other chunk. This chunk runs from the starting index
        // in the lexer to the beginning of the match.
        trace!(
            slog_scope::logger(),
            "Pushing token type {:?} containing {:?}",
            TokenType::Other,
            &doc_input[lexer.start..mat.start()],
        );
        lex.push(Token::new(
            TokenType::Other,
            &doc_input[lexer.start..mat.start()],
        ));

        // The other chunk is followed by either a footnote reference or the end
        // of the string. Unless the other chunk ends the string, the next chunk
        // is a footnote reference. It runs from the start of the match to the
        // end of the match.
        trace!(
            slog_scope::logger(),
            "Pushing token type {:?} containing {:?}",
            TokenType::FootnoteRef,
            &doc_input[mat.start()..mat.end()],
        );
        lex.push(Token::new(
            TokenType::FootnoteRef,
            &doc_input[mat.start()..mat.end()],
        ));

        // Set the new starting index.
        lexer.start = mat.end();
    }

    // After the last footnote-reference chunk is processed, there should still
    // be an other chunk. This closes that last chunk off.
    trace!(
        slog_scope::logger(),
        "Pushing token type {:?} containing {:?}",
        TokenType::Other,
        &doc_input[lexer.start..],
    );
    lex.push(Token::new(TokenType::Other, &doc_input[lexer.start..]));

    debug!(slog_scope::logger(), "Document lexing finished.");
    Ok(lex)
}

/// Lex the contents of `footnotes.xml`.
///
/// This function lexes the `footnotes.xml` contents into `CrossRef` and `Other`
/// tokens. It is probably a little brittle. It uses regex to find the
/// cross-references and more regex when there is a range of numbers. It then
/// relies on index offsets to identify the numbers.
///
/// The file should always start with an "other" chunk. And the loop always ends
/// with a new "other" chunk. So each loop should start by closing off an
/// "other."
fn lex_fn(input: &str) -> Result<Vec<Token>, String> {
    // Create a new lexer and empty vector of tokens
    let mut lexer = Lexer::new();
    let mut lex: Vec<Token> = Vec::new();

    // Use regex to identify each match.
    //
    // The first group `((>note )([0-9]{1,9}))` captures references to single
    // footnotes. It should have three total capture groups. The second group
    // `((>notes )([0-9]{1,9})(-|–)([0-9]{1,9}))` captures references to a range
    // of footnotes. It should have five total capture groups.
    let re =
        Regex::new(r#"((>note )([0-9]{1,9}))|((>notes )([0-9]{1,9})(-|–)([0-9]{1,9}))"#).unwrap();

    // This regex finds numbers within a range.
    let re_range = Regex::new(r#"([0-9]{1,9})(-|–)([0-9]{1,9})"#).unwrap();

    // Iterate over the matches groups
    for mat in re.find_iter(input) {
        // Determine whether the match is to a single footnote or a range. Note,
        // a hyphen is in the first conditional, an en-dash (U+2013) is in the
        // second.
        if mat.as_str().contains('-') || mat.as_str().contains('–') {
            // Push the precedeing "other" chunk, which goes from the lexer's
            // current starting index to seven spaces after the beginning of the
            // match.
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {:?}",
                TokenType::Other,
                &input[lexer.start..mat.start() + 7],
            );
            lex.push(Token::new(
                TokenType::Other,
                &input[lexer.start..mat.start() + 7],
            ));

            // Find the two numbers in the string.
            let range = re_range.captures(mat.as_str()).unwrap();

            // Then get their indexes. The first number starts at mat.start() +
            // 7 and ends at mat.start() + 7 + the length of the number
            let first_digit = (mat.start() + 7, mat.start() + 7 + range[1].len());
            // Then the range indicator, which should be an en-dash. I also
            // account for hyphens.
            let dash = (first_digit.1, first_digit.1 + range[2].len());
            // Then the second digit, which follows the range indicator and goes
            // to the end of that capture.
            let second_digit = (dash.1, dash.1 + range[3].len());

            // Then push the first number, the range indicator, and the second
            // number
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {}",
                TokenType::CrossRef,
                &input[first_digit.0..first_digit.1],
            );
            lex.push(Token::new(
                TokenType::CrossRef,
                &input[first_digit.0..first_digit.1],
            ));

            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {}",
                TokenType::Other,
                &input[dash.0..dash.1],
            );
            lex.push(Token::new(TokenType::Other, &input[dash.0..dash.1]));

            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {}",
                TokenType::CrossRef,
                &input[second_digit.0..second_digit.1],
            );
            lex.push(Token::new(
                TokenType::CrossRef,
                &input[second_digit.0..second_digit.1],
            ));

            // Set the new starting index
            lexer.start = mat.end();
        } else {
            // Push the precedeing "other" chunk, which goes from the lexer's
            // current starting index to six spaces after the beginning of the
            // match.
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {:?}",
                TokenType::Other,
                &input[lexer.start..mat.start() + 6],
            );
            lex.push(Token::new(
                TokenType::Other,
                &input[lexer.start..mat.start() + 6],
            ));

            // If there's no range of cross-references, then the "other" chunk
            // is followed by either a cross-reference or the end of the string.
            // Unless the "other" chunk ends the string, the next chunk is a
            // cross reference. It consists only of the number and thus runs
            // from six after the start of the match to the end of the match.
            trace!(
                slog_scope::logger(),
                "Pushing token type {:?} containing {:?}",
                TokenType::CrossRef,
                &input[mat.start() + 5..mat.end()],
            );
            lex.push(Token::new(
                TokenType::CrossRef,
                &input[mat.start() + 6..mat.end()],
            ));

            // Set the new starting index
            lexer.start = mat.end();
        }
    }

    // After the last cross-reference chunk is processed, there should still be
    // one last "other" chunk. Close that last chunk off.
    trace!(
        slog_scope::logger(),
        "Pushing token type {:?} containing {:?}",
        TokenType::Other,
        &input[lexer.start..],
    );
    lex.push(Token::new(TokenType::Other, &input[lexer.start..]));

    debug!(slog_scope::logger(), "Footnote lexing finished.");
    Ok(lex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_ref() {
        let input = r#"<w:footnote w:id="21"><w:p><w:pPr><w:pStyle w:val="FootnoteText" /></w:pPr><w:r>
  <w:rPr>
    <w:rStyle w:val="FootnoteReference" />
  </w:rPr>
  <w:footnoteRef />
</w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">Footnote 2. Cross references footnote 1.</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">See</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">supra</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">note 1.</w:t></w:r></w:p></w:footnote>"#;

        let tokens = lex_fn(input).unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TokenType::Other);
        assert_eq!(tokens[1].token_type, TokenType::CrossRef);
        assert_eq!(tokens[2].token_type, TokenType::Other);

        assert_eq!(
            tokens[0].contents,
            r#"<w:footnote w:id="21"><w:p><w:pPr><w:pStyle w:val="FootnoteText" /></w:pPr><w:r>
  <w:rPr>
    <w:rStyle w:val="FootnoteReference" />
  </w:rPr>
  <w:footnoteRef />
</w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">Footnote 2. Cross references footnote 1.</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">See</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">supra</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">note "#
        );
        assert_eq!(tokens[1].contents, "1");
        assert_eq!(tokens[2].contents, r#".</w:t></w:r></w:p></w:footnote>"#);
    }

    #[test]
    fn ref_range() {
        let input = r#"<w:footnote w:id="22"><w:p><w:pPr><w:pStyle w:val="FootnoteText" /></w:pPr><w:r>
  <w:rPr>
    <w:rStyle w:val="FootnoteReference" />
  </w:rPr>
  <w:footnoteRef />
</w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">Footnote 3. Cross references a range of footnotes, 1 and 2.</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">See</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">supra</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">notes 1–2.</w:t></w:r></w:p></w:footnote>"#;

        let tokens = lex_fn(input).unwrap();
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::Other);
        assert_eq!(tokens[1].token_type, TokenType::CrossRef);
        assert_eq!(tokens[2].token_type, TokenType::Other);
        assert_eq!(tokens[3].token_type, TokenType::CrossRef);
        assert_eq!(tokens[4].token_type, TokenType::Other);

        assert_eq!(
            tokens[0].contents,
            r#"<w:footnote w:id="22"><w:p><w:pPr><w:pStyle w:val="FootnoteText" /></w:pPr><w:r>
  <w:rPr>
    <w:rStyle w:val="FootnoteReference" />
  </w:rPr>
  <w:footnoteRef />
</w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">Footnote 3. Cross references a range of footnotes, 1 and 2.</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">See</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:rPr><w:iCs /><w:i /></w:rPr><w:t xml:space="preserve">supra</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">notes "#
        );
        assert_eq!(tokens[1].contents, r#"1"#);
        assert_eq!(tokens[2].contents, r#"–"#);
        assert_eq!(tokens[3].contents, r#"2"#);
        assert_eq!(tokens[4].contents, r#".</w:t></w:r></w:p></w:footnote>"#);
    }
}
