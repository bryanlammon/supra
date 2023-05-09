//! Module for replacing the spaces after footnote numbers with tabs.

use regex::Regex;
use slog::debug;

pub fn tabbed_footnotes(mut fns: String) -> Result<String, String> {
    debug!(slog_scope::logger(), "Tabbing footnotes...");

    // Setup the find and replace strings
    let find = r#"\n</w:r><w:r><w:t xml:space="preserve"> </w:t></w:r>"#;
    let replace = r#"</w:r><w:r><w:tab/></w:r>"#;

    // Setup the regex
    let re = Regex::new(find).unwrap();

    // Replace all of the spaces with tabs
    fns = re.replace_all(&fns, replace).to_string();

    debug!(slog_scope::logger(), "Footnotes tabbed.");
    Ok(fns)
}

pub fn tabbed_author_footnote(mut fns: String) -> Result<String, String> {
    debug!(slog_scope::logger(), "Tabbing the author footnote...");

    let au_find = r#"</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t>"#;
    let au_replace = r#"</w:t></w:r><w:r><w:tab/>"#;

    let re = Regex::new(au_find).unwrap();

    fns = re.replace(&fns, au_replace).to_string();

    debug!(slog_scope::logger(), "Author footnote tabbed.");
    Ok(fns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab() {
        let input = "\n</w:r><w:r><w:t xml:space=\"preserve\"> </w:t></w:r>".to_string();
        let output = tabbed_footnotes(input).unwrap();
        assert_eq!(&output, "</w:r><w:r><w:tab/></w:r>");
    }
}
