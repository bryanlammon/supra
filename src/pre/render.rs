//! This module contains functionality for rendering the output.

use crate::pre::{
    parser::{Branch, PreCite},
    sourcemap::{SourceMap, SourceType},
};
use slog::{debug, trace};
use std::collections::HashMap;

/// For tracking the last citation.
///
/// TODO This isn't working perfectly, as it reads a string cite to the same source as a string cite of multiple sources.
/// Maybe track the last citation clause, with all of the sources cited in it? A clause starts with a citation and ends once there is a `Text` branch that contains more than a blank space. It could count the number of citations and the sources cited (with no duplication). Then, the next citation would ask whether (1)\ there was a string cite and (2)\ whether it contained different sources. If both are true, an *id.* cannot be used.
struct LastCitation {
    footnote: i32,
    source: Option<String>,
    punctuation: char,
    string: bool,
}

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
    let mut last_citation = LastCitation {
        footnote: current_footnote,
        source: None,
        punctuation: ' ',
        string: false,
    };

    for branch in tree {
        output.push_str(&render_branch(
            branch,
            source_map,
            crossref_map,
            &mut current_footnote,
            &mut last_citation,
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
    last_citation: &mut LastCitation,
) -> String {
    trace!(slog_scope::logger(), "Rendering branch...");

    // Render and return based on the branch type
    match branch {
        Branch::Text(text) => text.contents.to_string(),
        Branch::Footnote(footnote) => {
            *current_footnote += 1;

            // Iterate through the branches of a footnote, collecting them in a
            // string.

            // TODO This should probably be a string with some capacity to avoid
            // reallocations.
            let mut contents: String = String::new();
            for footnote_branch in &footnote.contents {
                contents.push_str(&render_branch(
                    footnote_branch,
                    source_map,
                    crossref_map,
                    current_footnote,
                    last_citation,
                ));
            }
            format!("^[{}]", contents.trim())
        }
        Branch::Citation(citation) => {
            let mut contents = String::new();
            let mut capitalize = true;

            // First render the pre-citation puncutation or signal.
            if let Some(r) = &citation.pre_cite {
                match r {
                    PreCite::Punctuation(p) => {
                        if !p.contents.contains('.')
                            && !p.contents.contains('!')
                            && !p.contents.contains('?')
                        {
                            capitalize = false;
                        }
                        contents.push_str(p.contents);
                    }
                    PreCite::Signal(s) => {
                        capitalize = false;
                        contents.push_str(s.contents);
                    }
                }
            }

            // Rendering a citation depends on the source type. If the source is
            // an article, book, chapter, or manuscript, all that matters is
            // whether the source has been cited. If the source is a case, we
            // need to know how far back the last cite was.

            if last_citation.source.is_some()
                && !last_citation.string
                && &source_map[citation.reference].id == last_citation.source.as_ref().unwrap()
            {
                // It's an *Id.*
                match capitalize {
                    true => contents.push_str("*Id.*"),
                    false => contents.push_str("*id.*"),
                }

                // Add a pin, if any.
                if citation.pincite.is_some() {
                    contents.push_str(" at ");
                    contents.push_str(citation.pincite.as_ref().unwrap());
                }
            } else if source_map[citation.reference].source_type == SourceType::Case {
                // Case citation.

                // Determine if this is the first footnote OR if the case has been cited in the last 4 footnotes.
                let current_footnote_local = *current_footnote;

                // Has the case been cited in the last five footnotes?
                let last_five = source_map[citation.reference]
                    .all_footnotes
                    .contains(&(current_footnote_local - 1))
                    || source_map[citation.reference]
                        .all_footnotes
                        .contains(&(current_footnote_local - 2))
                    || source_map[citation.reference]
                        .all_footnotes
                        .contains(&(current_footnote_local - 3))
                    || source_map[citation.reference]
                        .all_footnotes
                        .contains(&(current_footnote_local - 4))
                    || source_map[citation.reference]
                        .all_footnotes
                        .contains(&(current_footnote_local - 5));

                // Or is it the first time its cited?
                if current_footnote_local == source_map[citation.reference].all_footnotes[0]
                    || !last_five
                {
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

            // Update the last citation to the what was just cited.
            // If the previous citation ended with a semicolon or comma, then the current citation is probably---though not definitely---part of a string.
            if last_citation.punctuation == ',' || last_citation.punctuation == ';' {
                last_citation.string = true;
            } else {
                last_citation.string = false;
            }

            last_citation.footnote = *current_footnote;
            last_citation.source = Some(source_map[citation.reference].id.clone());
            last_citation.punctuation = citation.punctuation.chars().next().unwrap();

            contents
        }

        Branch::CrossRef(crossref) => crossref_map[crossref.contents].to_string(),

        Branch::CiteBreak => {
            last_citation.source = None;
            "".to_string()
        }
    }
}
