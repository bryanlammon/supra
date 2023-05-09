//! This module contains functionality for rendering the output.

use crate::pre::{
    parser::{Branch, PreCite},
    sourcemap::{SourceMap, SourceType},
};
use ansi_term::Color;
use slog::{debug, trace};
use std::collections::HashMap;

/// For tracking the last citations.
///
/// This tracks the last citation clause, adding sources to the collection until
/// it hits a period, which closes off the clause. After a clause is closed off,
/// the next citation will ask whether the previous clause included more than
/// one citation. If it did, then an `*Id.*` is not appropriate. If it didn't,
/// the next citation will ask whether it was the only source cited in the last
/// clause. If it was, then an `*Id.*` is appropriate.
struct LastCitation {
    sources: Vec<String>,
    closed: bool,
    last_pin: String,
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
        sources: Vec::new(),
        closed: false,
        last_pin: "".to_string(),
    };

    // Push the render results.
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

/// Renders a branch.
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

            // TODO This should probably be a string with some capacity to avoid
            // reallocations.
            let mut contents: String = String::new();

            // Iterate through the branches of a footnote, collecting them in a
            // string.
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
            let mut render_punctuation = true;

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

            // If (1)\ (a)\ the last citation clause is finished, (b)\ it
            // contained only one source, and (c)\ that source is the same as
            // the current one, OR (2)\ (a)\ the last citation clause is open,
            // and (b)\ the immediately preceding source is the same one, use an
            // `*Id.*`.
            if (last_citation.closed
                && last_citation.sources.len() == 1
                && source_map[citation.reference].id == last_citation.sources[0])
                || (!last_citation.closed
                    && !last_citation.sources.is_empty()
                    && &source_map[citation.reference].id == last_citation.sources.last().unwrap())
            {
                // It's an *Id.*
                match capitalize {
                    true => contents.push_str("*Id.*"),
                    false => contents.push_str("*id.*"),
                }

                // If there's nothing after the *Id.*, don't render any
                // subsequent punctuation.
                render_punctuation = false;

                // If there's a pincite AND it's different from the last
                // pincite, add a pin.
                if citation.pincite.is_some()
                    && citation.pincite.as_ref().unwrap() != &last_citation.last_pin
                {
                    contents.push_str(" at ");
                    contents.push_str(citation.pincite.as_ref().unwrap());

                    //last_citation.last_pin =
                    //citation.pincite.as_ref().unwrap().to_string();

                    // There's something between the *Id.* and the punctuation,
                    // so the period in the *Id.* can't be the ending
                    // punctuation.
                    render_punctuation = true;
                }
            } else if source_map[citation.reference].source_type == SourceType::Case {
                // Case citation.

                // Determine if this is the first footnote OR if the case has
                // been cited in the last 4 footnotes.
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

            // Regardless of the cite type.
            if citation.parenthetical.is_some() {
                contents.push(' ');
                contents.push_str(citation.parenthetical.unwrap());

                render_punctuation = true;
            }

            // Then add the punctuation unless (1)\ (a)\ the cite was an *Id.*,
            // and (b)\ there was no pincite or parenthetical, OR (2)\ the
            // puncutation is not `.`.
            if render_punctuation || citation.punctuation != "." {
                contents.push_str(citation.punctuation)
            };

            // Update the last citation to what was just cited.
            //
            // First, if the previous clause is closed, erase it and start a new
            // one. If it's not, add the new citation to the collection if it's
            // not already there.
            if last_citation.closed {
                last_citation.sources = vec![source_map[citation.reference].id.clone()];
                last_citation.closed = false;
            } else if !last_citation
                .sources
                .contains(&source_map[citation.reference].id)
            {
                last_citation
                    .sources
                    .push(source_map[citation.reference].id.clone());
            }

            // If there was a pincite, update that.
            if citation.pincite.is_some() {
                last_citation.last_pin = citation.pincite.as_ref().unwrap().to_string();
            }

            // Then, if the latest's sources punctuation ends a clause, close
            // off the clause.
            if citation.punctuation == "."
                || citation.punctuation == "!"
                || citation.punctuation == "?"
            {
                last_citation.closed = true;
            }

            contents
        }

        Branch::CrossRef(crossref) => {
            if crossref_map.contains_key(crossref.contents) {
                crossref_map[crossref.contents].to_string()
            } else {
                eprintln!(
                    "{} Pre-processing error: no footnote with the id \"{}\"",
                    Color::Red.paint("ERRO"),
                    crossref.contents
                );
                crossref.contents.to_string()
            }
        }

        Branch::CiteBreak => {
            last_citation.sources = Vec::new();
            last_citation.closed = true;
            "".to_string()
        }
    }
}
