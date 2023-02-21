//! This module contains the functionality for determing the first bookmark id
//! to use for cross-references.

use regex::Regex;
use slog::debug;

/// Determine the bookmark id number to start with.
///
/// The `document.xml` might already have bookmarks in it referring to the
/// headings or other items. Those bookmarks seem to come with arbitrary ids. So
/// we must be careful to ensure that no bookmark idsare duplicated.
///
/// This function collects all of the bookmark ids in the provided string (the
/// `document.xml` file). It then adds 1 to the highest number and returns that
/// number.
pub fn starting_bookmark(doc_input: &str) -> Result<u32, String> {
    debug!(slog_scope::logger(), "Determining starting bookmark id...");

    // Create a new vector for storing all of the existing bookmarks
    let mut all_bookmarks: Vec<u32> = Vec::new();

    // Use regex to get all of the bookmarks in the provided string
    let re = Regex::new(r#"(<w:bookmarkStart w:id=")([0-9]{1,9})"#).unwrap();
    for cap in re.captures_iter(doc_input) {
        match cap[2].parse::<u32>() {
            Ok(b) => all_bookmarks.push(b),
            Err(e) => {
                let err_msg = format!("Error parsing existing bookmarks in document.xml: {}", e);
                return Err(err_msg);
            }
        }
    }
    match all_bookmarks.iter().max() {
        Some(i) => {
            debug!(slog_scope::logger(), "Starting bookmark is {}", i + 1);
            Ok(i + 1)
        }
        None => {
            debug!(slog_scope::logger(), "Starting bookmark is {}", 1);
            Ok(1)
        }
    }
}
