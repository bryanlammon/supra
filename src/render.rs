//! This module contains functionality for rendering the output.

use crate::{parser::Branch, sourcemap::SourceMap};
use slog::{debug, trace};
use std::collections::HashMap;

/// The main render function.
///
/// Iterates through the branches and sends each to the [`render_branch`]
/// function to a a string in return. Sending to a separate function is
/// necessary because some branches contain sub-branches, which themselves need
/// to be rendered/send to the [`render_branch`] function.
pub fn render(
    tree: &[Branch],
    source_map: &mut SourceMap,
    crossref_map: &HashMap<&str, i32>,
) -> String {
    debug!(slog_scope::logger(), "Beginning rendering...");

    // TODO This should probably be a string with some capacity to avoid
    // reallocations.
    let mut output = String::new();
    for branch in tree {
        output.push_str(&render_branch(branch, source_map, crossref_map));
    }

    debug!(slog_scope::logger(), "Rendering complete");
    output
}

/// Renders an individual branch.
///
/// Receives branches and renders them depending on their type, returning a
/// string. Note that branches themselves can call this funciton to render their
/// contents (which footnote branches do).
fn render_branch(
    branch: &Branch,
    source_map: &mut SourceMap,
    crossref_map: &HashMap<&str, i32>,
) -> String {
    trace!(slog_scope::logger(), "Rendering branch...");

    // Render and return based on the branch type
    match branch {
        Branch::Text(text) => text.contents.to_string(),
        Branch::Footnote(footnote) => {
            // Iterate through the branches of a footnote, collecting them in a
            // string. TODO This should probably be a string with some capacity
            // to avoid reallocations.
            let mut contents: String = String::new();
            for footnote_branch in &footnote.contents {
                contents.push_str(&render_branch(footnote_branch, source_map, crossref_map));
            }
            format!("^[{}]", contents.trim())
        }
        Branch::Citation(citation) => {
            let mut contents = String::from(" ");

            if !source_map[citation.reference].cited {
                source_map.get_mut(citation.reference).unwrap().cited = true;
                contents.push_str(&source_map[citation.reference].long_cite);
            } else {
                contents.push_str(&source_map[citation.reference].short_cite);

                if citation.pincite.is_some() {
                    contents.push_str(", at ");
                    contents.push_str(citation.pincite.unwrap());
                }
            }

            if citation.parenthetical.is_some() {
                contents.push(' ');
                contents.push_str(citation.parenthetical.unwrap());
            }

            contents.push_str(citation.punctuation);

            contents
        }

        Branch::CrossRef(crossref) => crossref_map[crossref.contents].to_string(),
    }
}
