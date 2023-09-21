//! Module for manually adding all-ligatures to "Th"s.

use regex::Regex;
use slog::debug;

/// Add discretionary ligatures to "Th" in the document.
///
/// EB Garamond does not add ligatures to "Th" in the normal ligatures.
/// Discretionary ligatures are thus necessary. But the entire document can't
/// have discretionary ligatures. So this function marks up the `document.xml`
/// file to make "Th" use all ligatures.
///
/// A schema must be added to the file:
/// `xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml`. Otherwise
/// the file will not work.
///
/// It looks like most of the "Th" in a Pandoc-produced document are preceded by
/// `<w:t xml:space="preserve>"`. A few---I think non-sentence-starting
/// uses---are preceded by a space. I can't find any that are preceeeded by any
/// other character (say, a `"` or `(`).
///
/// This function could be expanded to add other ligatures, if needed.
pub fn ligatures(mut doc: String, mut fns: String) -> Result<(String, String), String> {
    debug!(slog_scope::logger(), "Adding ligatures...");

    // Add the necessary schema to both the document and footnotes.
    let find = "<w:document";
    let replace = r#"<w:document xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
    let re = Regex::new(find).unwrap();
    doc = re.replace(&doc, replace).to_string();

    let find = "<w:footnotes";
    let replace =
        r#"<w:footnotes xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
    let re = Regex::new(find).unwrap();
    fns = re.replace(&fns, replace).to_string();

    // "Th" at the beginning of a section.
    let find = r#"<w:t xml:space="preserve">Th"#;
    let replace = r#"<w:rPr><w14:ligatures w14:val="all"/></w:rPr><w:t>Th</w:t></w:r><w:r><w:t xml:space="preserve">"#;
    let re = Regex::new(find).unwrap();
    doc = re.replace_all(&doc, replace).to_string();

    // This is probably going to error, because footnotes have so much more markup in them....
    //
    // This needs more work to deal with all of the extra formatting markup in footnotes.
    let find = r#"<w:t>Th"#;
    let re = Regex::new(find).unwrap();
    fns = re.replace_all(&fns, replace).to_string();

    // Second find & replace: "Th" in the middle of a section.
    //
    //let find = r#"(?<pre>[^>])Th"#; let replace =
    //r#"$pre</w:t></w:r><w:r><w:rPr><w14:ligatures
    //w14:val="all"/></w:rPr><w:t>Th</w:t></w:r><w:r><w:t
    //xml:space="preserve">"#; let re = Regex::new(find).unwrap(); doc =
    //re.replace_all(&doc, replace).to_string();

    // This was the second attempt at getting "Th" that have a space before
    // them. It didn't work.
    //
    //let find = " Th"; let replace = r#"</w:t></w:r><w:r><w:rPr><w14:ligatures
    //w14:val="all"/></w:rPr><w:t>Th</w:t></w:r><w:r><w:t
    //xml:space="preserve""#; let re = Regex::new(find).unwrap(); doc =
    //re.replace_all(&doc, replace).to_string();

    // Third find & replace: unnecessary spacing added by the previous find &
    // replace.
    //
    //let find = r#"<w:t xml:space="preserve"> </w:t>"#; let replace =
    //r#"<w:t xml:space="preserve"></w:t>"#; let re = Regex::new(find).unwrap();
    //doc = re.replace_all(&doc, replace).to_string();

    debug!(slog_scope::logger(), "\"Th\" ligatures added.");

    Ok((doc, fns))
}
