//! This module contains functionality for rendering the output.

use crate::pre::{
    parser::Branch,
    sourcemap::{SourceMap, SourceType},
};
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

    // Track the footnotes and cites.
    let mut current_footnote = 0;

    for branch in tree {
        output.push_str(&render_branch(
            branch,
            source_map,
            crossref_map,
            &mut current_footnote,
        ));
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
    current_footnote: &mut i32,
) -> String {
    trace!(slog_scope::logger(), "Rendering branch...");

    // Render and return based on the branch type
    match branch {
        Branch::Text(text) => text.contents.to_string(),
        Branch::Footnote(footnote) => {
            // Iterate through the branches of a footnote, collecting them in a
            // string.

            *current_footnote += 1;

            // TODO This should probably be a string with some capacity to avoid
            // reallocations.
            let mut contents: String = String::new();
            for footnote_branch in &footnote.contents {
                contents.push_str(&render_branch(
                    footnote_branch,
                    source_map,
                    crossref_map,
                    current_footnote,
                ));
            }
            format!("^[{}]", contents.trim())
        }
        Branch::Citation(citation) => {
            let mut contents = String::from(" ");

            // Rendering a citation depends on the source type. If the source is
            // an article, book, chapter, or manuscript, all that matters is
            // whether the source has been cited. If the source is a case, we
            // need to know how far back the last cite was. It's probably best
            // to start counting the footnotes, which might make `Id.`s easier.

            if source_map[citation.reference].source_type == SourceType::Case {
                // Case citation.

                // Determine if this is the first footnote.
                if *current_footnote == source_map[citation.reference].all_footnotes[0] {
                    // Long form.
                    if citation.pincite.is_some() {
                        contents.push_str(
                            &source_map[citation.reference]
                                .long_cite_w_pin(citation.pincite.unwrap()),
                        );
                    } else {
                        contents.push_str(&source_map[citation.reference].long_cite_no_pin());
                    }
                } else {
                    // Short form.
                    if citation.pincite.is_some() {
                        contents.push_str(
                            &source_map[citation.reference]
                                .short_cite_w_pin(citation.pincite.unwrap()),
                        );
                    } else {
                        contents.push_str(&source_map[citation.reference].short_cite_no_pin())
                    }
                }
            } else if !source_map[citation.reference].cited {
                // Citation to a source using supra that has not yet been cited.

                // Set the cited flag and output a long cite.
                source_map.get_mut(citation.reference).unwrap().cited = true;

                if citation.pincite.is_some() {
                    contents.push_str(
                        &source_map[citation.reference].long_cite_w_pin(citation.pincite.unwrap()),
                    );
                } else {
                    contents.push_str(&source_map[citation.reference].long_cite_no_pin());
                }
            } else if citation.pincite.is_some() {
                // Otherwise the source using a supra has already been cited.
                contents.push_str(
                    &source_map[citation.reference].short_cite_w_pin(citation.pincite.unwrap()),
                );
            } else {
                contents.push_str(&source_map[citation.reference].short_cite_no_pin());
            }

            // Reardless of the cite type, add the parenthetical and punctuation.
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
