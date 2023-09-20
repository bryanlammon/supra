//! Module for changing the running headers.
use regex::Regex;
use slog::debug;

pub fn running_header(
    mut h1: String,
    mut h2: String,
    running_header: String,
    year: String,
) -> Result<(String, String), String> {
    debug!(slog_scope::logger(), "Editing headers...");

    let header_find = r"\[.+?\]";
    let header_replace = running_header;

    let re1 = Regex::new(header_find).unwrap();
    h1 = re1.replace(&h1, header_replace.clone()).to_string();
    h2 = re1.replace(&h2, header_replace).to_string();

    let year_find = r#"date"#;
    let year_replace = year;

    let re2 = Regex::new(year_find).unwrap();

    h2 = re2.replace(&h2, year_replace).to_string();

    debug!(slog_scope::logger(), "Headers edited.");
    Ok((h1, h2))
}
