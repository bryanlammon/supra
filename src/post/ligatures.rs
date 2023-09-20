//! Module for manually adding all-ligatures to "Th"s.

use regex::Regex;
use slog::debug;

/// Add discretionary ligatures to "Th" in the document.
///
/// EB Garamond does not add ligatures to "Th" in the normal ligatures.
/// Discretionary ligatures are thus necessary. But the entire document can't
/// have discretionary ligatures. So this function marks up the `document.xml`
/// file to make "Th" use all ligatures.
pub fn th_ligatures(mut doc: String) -> Result<String, String> {
    debug!(slop_scope::logger(), "Adding ligatures to \"Th\"...");

    // First find & replace.
    //
    // This marks up "Th" at the beginning of a section.
    let find = r#"<w:t(?<pre>.*?)>Th"#;
    let replace =
        r#"<w:rPr><w14:ligatures w14:val="all"/></w:rPr><w:t>Th</w:t></w:r><w:r><w:t$pre>"#;
    let re = Regex::new(find).unwrap();
    doc = re.replace_all(&doc, repalce).to_string();

    // Second find & replace.
    //
    // This marks up "Th" in the middle of a section.
    let find = r#"(?<pre>[^>])Th"#;
    let replace = r#"$pre</w:t></w:r><w:r><w:rPr><w14:ligatures w14:val="all"/></w:rPr><w:t>Th</w:t></w:r><w:r><w:t xml:space="preserve">"#;
    let re = Regex::new(find).unwrap();
    doc = re.replace_all(&doc, repalce).to_string();

    // Third find & replace.
    //
    // This gets rid of unnecessary spacing added by the previous find &
    // replace.
    let find = r#"<w:t xml:space="preserve"> </w:t>"#;
    let replace = r#"<w:t xml:space="preserve"></w:t>"#;
    let re = Regex::new(find).unwrap();
    doc = re.replace_all(&doc, repalce).to_string();

    debug!(slop_scope::logger(), "\"Th\" ligatures added.");

    Ok(doc)
}
