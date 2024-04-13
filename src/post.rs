//! The Supra post-processor.
mod author_note;
mod autocref;
mod docx;
mod no_superscript;
mod running_header;
mod tabbed_footnotes;

use serde::Deserialize;
use slog::{debug, o};
use std::path::Path;

/// Metadata for markup
#[derive(Debug, Deserialize)]
struct Metadata {
    #[allow(dead_code)]
    title: Option<String>,
    author: Option<String>,
    author_note: Option<String>,
    year: Option<String>,
    running_header: Option<String>,
}

/// The post-processor.
pub fn post(
    md_input: &str,
    output: &Path,
    autocref: bool,
    author_note: bool,
    tabbed_footnotes: bool,
    no_superscript: bool,
    running_header: bool,
) -> Result<(), String> {
    debug!(slog_scope::logger(), "Starting post-processor...");
    // Get any necessary metadata
    let mut metadata: Metadata = Metadata {
        title: None,
        author: None,
        author_note: None,
        year: None,
        running_header: None,
    };
    if author_note || running_header {
        // Get the first document, which has the yaml data
        if let Some(document) = serde_yaml::Deserializer::from_str(md_input).next() {
            if let Ok(m) = Metadata::deserialize(document) {
                metadata = m;
            } else {
                let err_msg = "Error reading yaml metadata";
                return Err(err_msg.to_string());
            }
        };
    };

    // Read document.xml and footnotes.xml from the .docx file
    let (mut doc, mut fns, mut h1, mut h2) =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "read_docx()")), || {
            docx::read_docx(output)
        }) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };

    // Run AutoCref
    if autocref {
        (doc, fns) =
            match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "autocref()")), || {
                autocref::autocref(doc, fns)
            }) {
                Ok(a) => a,
                Err(e) => return Err(e),
            };
    }

    // Author-note function
    if author_note {
        if metadata.author.is_some() && metadata.author_note.is_some() {
            (doc, fns) =
                match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "author()")), || {
                    author_note::author_note(
                        doc,
                        fns,
                        metadata.author.unwrap(),
                        metadata.author_note.unwrap(),
                    )
                }) {
                    Ok(a) => a,
                    Err(e) => return Err(e),
                };
        } else {
            return Err("Insufficient metadata for author note".to_string());
        }
    }

    // Tabbed-footnotes function
    if tabbed_footnotes {
        fns = match slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "tabbed_footnotes()")),
            || tabbed_footnotes::tabbed_footnotes(fns),
        ) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        // Run the special author-note function if one was added
        if author_note {
            fns = match slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "tabbed_author_footnote()")),
                || tabbed_footnotes::tabbed_author_footnote(fns),
            ) {
                Ok(f) => f,
                Err(e) => return Err(e),
            };
        }
    }

    // No-superscript function
    if no_superscript {
        fns = match slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "no_superscript()")),
            || no_superscript::no_superscript(fns),
        ) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
    }

    // Running-header function
    if running_header {
        if !&h1.is_empty() && !&h2.is_empty() {
            if metadata.running_header.is_some() && metadata.year.is_some() {
                (h1, h2) = match slog_scope::scope(
                    &slog_scope::logger().new(o!("fn" => "running_header()")),
                    || {
                        running_header::running_header(
                            h1,
                            h2,
                            metadata.running_header.unwrap(),
                            metadata.year.unwrap(),
                        )
                    },
                ) {
                    Ok(f) => f,
                    Err(e) => return Err(e),
                };
            } else {
                return Err("Insufficient metadata for running header".to_string());
            }
        } else {
            return Err("Custom reference does not contain headers".to_string());
        }
    }

    // Write the .docx file
    match slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "write_docx()")),
        || docx::write_docx(output, doc, fns, h1, h2),
    ) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };

    debug!(slog_scope::logger(), "Post-processing compeleted.");
    Ok(())
}
