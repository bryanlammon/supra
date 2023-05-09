use slog::debug;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
use zip::{write, CompressionMethod, ZipArchive, ZipWriter};

/// Read the `.docx` file.
///
/// This function takes the path to the `.docx` file and reads the
/// `document.xml`, `footnotes.xml`, `header1.xml`, and `header2.xml` files,
/// outputting their contents as strings.
pub fn read_docx(path: &Path) -> Result<(String, String, String, String), String> {
    debug!(slog_scope::logger(), "Reading {:?}...", path);

    // Load the .path file
    let docx_file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(e.to_string()),
    };

    // Create a ZipArchive from the .docx file
    let mut docx = match ZipArchive::new(docx_file) {
        Ok(z) => z,
        Err(e) => return Err(e.to_string()),
    };

    // Read the document.xml and footnotes.xml files
    let mut doc = String::new();
    docx.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut doc)
        .unwrap();

    let mut fns = String::new();
    docx.by_name("word/footnotes.xml")
        .unwrap()
        .read_to_string(&mut fns)
        .unwrap();

    let mut h1 = String::new();
    if let Ok(mut z) = docx.by_name("word/header1.xml") {
        z.read_to_string(&mut h1).unwrap();
    } else {
        h1 = "".to_string();
    }

    let mut h2 = String::new();
    if let Ok(mut z) = docx.by_name("word/header2.xml") {
        z.read_to_string(&mut h2).unwrap();
    } else {
        h2 = "".to_string();
    }

    debug!(slog_scope::logger(), "{:?} read.", path);
    Ok((doc, fns, h1, h2))
}

/// Write the new `.docx` file.
///
/// This function starts by recreating the ZipArchive used in [`read_docx`]
/// (needed because that variable is dropped after reading). It then creates the
/// output file, replacing the contents of `document.xml`, `footnotes.xml`,
/// `header1.xml`, and `header2.xml`.
pub fn write_docx(
    path: &Path,
    doc: String,
    fns: String,
    h1: String,
    h2: String,
) -> Result<(), String> {
    debug!(slog_scope::logger(), "Writing {:?}...", path);
    // Load the .docx file
    let docx_file = std::fs::File::open(path).unwrap();

    // Create a ZipArchive from the .docx file
    let mut docx = ZipArchive::new(docx_file).unwrap();

    // Delete the old file so it can be overwritten
    debug!(slog_scope::logger(), "Deleting old {:?}...", path);
    match fs::remove_file(path) {
        Ok(_) => (),
        Err(e) => return Err(format!("Cannot overwrite .docx ({:?})", e)),
    }

    // Create a ZipWriter and its options (.docx compression is Deflated)
    let output_file = std::fs::File::create(path).unwrap();
    let mut output = ZipWriter::new(output_file);
    let options = write::FileOptions::default().compression_method(CompressionMethod::Deflated);

    // Iterate through the docx contents, replacing as necessary
    for i in 0..docx.len() {
        // Get the file
        let mut file = docx.by_index(i).unwrap();

        // Start writing it
        let _ = output.start_file(file.name(), options);

        let mut contents = String::new();
        let contents_b: &[u8];

        // Determine what to write
        if file.name() == "word/document.xml" {
            // If it's document.xml, use the contents of doc
            contents_b = doc.as_bytes();
        } else if file.name() == "word/footnotes.xml" {
            // If it's footnotes.xml, use the contents of fn
            contents_b = fns.as_bytes();
        } else if file.name() == "word/header1.xml" {
            // If it's header1.xml, use the contents of h1
            contents_b = h1.as_bytes();
        } else if file.name() == "word/header2.xml" {
            // If it's header2.xml, use the contents of h2
            contents_b = h2.as_bytes();
        } else {
            // Anything else, rewrite contents of the original
            file.read_to_string(&mut contents).unwrap();
            contents_b = contents.as_bytes();
        }
        let _ = output.write_all(contents_b);
    }

    // Finish writing the zip file
    let _ = output.finish();

    debug!(slog_scope::logger(), "{:?} written.", path);
    Ok(())
}
