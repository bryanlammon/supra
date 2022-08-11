//! This module contains functionality for output options.

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Regex for bold text
    pub static ref BOLD: Regex = Regex::new(r"\*\*(?P<input>.+?)\*\*").unwrap();
}

/// Change any bolded text to the Word custom style "True Small Caps"
pub fn smallcaps(input: &str) -> String {
    BOLD.replace_all(input, "[$input]{custom-style=\"True Small Caps\"}")
        .to_string()
}
