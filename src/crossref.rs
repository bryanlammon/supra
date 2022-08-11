//! This module contains functions for building the cross-reference map.

use crate::parser::Branch;
use slog::debug;
use std::collections::HashMap;

/// Creates the cross-reference map.
///
/// Iterates through the branches looking for footnotes. If a footnote has an
/// id, the id and footnote number are added to the map.
pub fn build_crossref_map<'a>(tree: &'a [Branch]) -> HashMap<&'a str, i32> {
    debug!(slog_scope::logger(), "Beginning crossref map build...");
    let mut crossref_map: HashMap<&str, i32> = HashMap::new();

    for branch in tree {
        if let Branch::Footnote(footnote) = branch {
            if footnote.id.is_some() {
                crossref_map.insert(footnote.id.unwrap(), footnote.number);
            }
        }
    }

    debug!(slog_scope::logger(), "Crossref map build complete");
    crossref_map
}
