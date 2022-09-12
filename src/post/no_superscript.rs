//! Module for making footnote numbers non-superscript.

use regex::Regex;
use slog::debug;

pub fn no_superscript(mut fns: String) -> Result<String, String> {
    debug!(slog_scope::logger(), "Removing superscripts...");

    // Setup the find and replace strings
    let find = r#"<w:rStyle w:val="FootnoteReference" />"#;
    let replace = r#"<w:rStyle w:val="FootnoteReference"/><w:vertAlign w:val="baseline"/>"#;

    // Setup the regex
    let re = Regex::new(find).unwrap();

    // Remove the superscripts
    fns = re.replace_all(&fns, replace).to_string();

    debug!(slog_scope::logger(), "Superscripts removed.");
    Ok(fns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_superscript() {
        let input = r#"<w:rStyle w:val="FootnoteReference" />"#.to_string();
        let output = no_superscript(input).unwrap();
        assert_eq!(
            &output,
            r#"<w:rStyle w:val="FootnoteReference"/><w:vertAlign w:val="baseline"/>"#
        );
    }
}
