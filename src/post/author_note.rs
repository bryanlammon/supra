use regex::Regex;

pub fn author_note(
    mut doc: String,
    mut fns: String,
    name: String,
    note: String,
) -> Result<(String, String), String> {
    // Add the star note to the author's name
    doc = match add_mark(doc, &name) {
        Ok(d) => d,
        Err(e) => {
            let err = format!("Error adding author-note marker: {:?}", e);
            return Err(err);
        }
    };

    // Add the contents of the author note
    fns = match add_note(fns, note) {
        Ok(f) => f,
        Err(e) => {
            let err = format!("Error adding author-note contents: {:?}", e);
            return Err(err);
        }
    };

    Ok((doc, fns))
}

fn add_mark(mut doc: String, name: &str) -> Result<String, String> {
    // First find the last name; if there's only one name, use that
    let last_name = name.split_whitespace().next_back().unwrap_or(name);

    // Then setup the regex strings
    let name_find = format!("{}</w:t></w:r>", last_name);
    let name_replace = format!("{}</w:t></w:r><w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\" /></w:rPr><w:footnoteReference w:customMarkFollows=\"1\" w:id=\"1\"/><w:t>*</w:t></w:r>", last_name);

    // Then setup the regex
    let re1 = Regex::new(&name_find).unwrap();

    // Finally, replace the first instance of the last name
    doc = re1.replace(&doc, name_replace).to_string();

    Ok(doc)
}

fn add_note(mut fns: String, note: String) -> Result<String, String> {
    let fns_find = r#"<w:footnote w:id="[1-9]\d*">"#.to_string();
    let fns_replace = format!("<w:footnote w:id=\"1\"><w:p><w:pPr><w:pStyle w:val=\"FootnoteText\"/></w:pPr><w:r><w:rPr><w:rStyle w:val=\"FootnoteReference\" /></w:rPr><w:t>*</w:t></w:r><w:r><w:t xml:space=\"preserve\"> </w:t></w:r><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p></w:footnote><w:footnote w:id=\"20\">", note);

    let re2 = Regex::new(&fns_find).unwrap();

    fns = re2.replace(&fns, fns_replace).to_string();

    Ok(fns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_mark() {
        let mut doc = r#"<w:r><w:t xml:space="preserve">Test</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">Author</w:t></w:r></w:p>"#.to_string();
        let name = "Author";

        doc = add_mark(doc, name).unwrap();

        assert_eq!(doc, r#"<w:r><w:t xml:space="preserve">Test</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">Author</w:t></w:r><w:r><w:rPr><w:rStyle w:val="FootnoteReference" /></w:rPr><w:footnoteReference w:customMarkFollows="1" w:id="1"/><w:t>*</w:t></w:r></w:p>"#.to_string())
    }

    #[test]
    fn test_add_note() {
        let mut fns = r#"<?xml version="1.0" encoding="UTF-8"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w10="urn:schemas-microsoft-com:office:word" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator /></w:r></w:p></w:footnote><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator /></w:r></w:p></w:footnote><w:footnote w:id="20">"#.to_string();
        let note = "Author Note Text".to_string();

        fns = add_note(fns, note).unwrap();

        assert_eq!(
            fns,
            r#"<?xml version="1.0" encoding="UTF-8"?><w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w10="urn:schemas-microsoft-com:office:word" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator /></w:r></w:p></w:footnote><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator /></w:r></w:p></w:footnote><w:footnote w:id="1"><w:p><w:pPr><w:pStyle w:val="FootnoteText"/></w:pPr><w:r><w:rPr><w:rStyle w:val="FootnoteReference" /></w:rPr><w:t>*</w:t></w:r><w:r><w:t xml:space="preserve"> </w:t></w:r><w:r><w:t xml:space="preserve">Author Note Text</w:t></w:r></w:p></w:footnote><w:footnote w:id="20">"#
        );
    }
}
