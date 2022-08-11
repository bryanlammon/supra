//! The module contains the functionality for creating the source map.

mod buildsource;

use super::csljson::CSLSource;
use super::parser::Branch;
use super::userjournals::UserJournals;
use ansi_term::Color;
use slog::{debug, o, trace, warn};
use std::collections::HashMap;

pub type SourceMap<'a> = HashMap<&'a str, Source>;

/// A source cited in the input document and built from the CSL JSON library.
#[derive(Debug)]
pub struct Source {
    pub id: String,
    pub source_type: SourceType,
    pub first_footnote: i32,
    pub all_footnotes: Vec<i32>,
    pub long_cite: String,
    pub short_cite: String,
    pub short_title: String,
    pub cited: bool,
    pub hereinafter: bool,
}

/// The types of currently supported sources, plus the catchall `Other`.
#[derive(Debug, PartialEq, Eq)]
pub enum SourceType {
    Book,
    Chapter,
    JournalArticle,
    Manuscript,
    Other,
}

/// Create the [`SourceMap`].
///
/// This function creates the [`SourceMap`], which is a hash map with ids as
/// keys and [`Source`]s as values. This [`SourceMap`] is then used in replacing
/// the citations in the Pandoc markdown document.
///
/// Creating the [`SourceMap`] requires three steps. It first creates most of
/// the information for each source. But full short cites cannot be determined
/// until after all sources have been collected. So the function must then
/// determine which sources need "hereinafters." After that, it creates
/// appropriate short cites.
///
/// Note, the identifiers in the [`SourceMap`] map include the brackets and at
/// symbol, *e.g.*, `[@smith2021]`. But the identifers in [`CSLSource`]s don't,
/// *e.g.*, `smith2021`.
pub fn build_source_map<'a>(
    tree: &'a [Branch],
    csl_library: Vec<CSLSource>,
    user_journals: Option<UserJournals>,
) -> SourceMap<'a> {
    debug!(slog_scope::logger(), "Starting source map...");

    // Start the source map.
    let mut source_map = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "start_source_map()")),
        || start_source_map(csl_library, tree, user_journals),
    );

    // Determine the need for hereinafters
    source_map = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "check_hereinafters()")),
        || check_hereinafters(source_map),
    );

    // Finish the source map with appropriate short cites
    source_map = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "add_short_cites()")),
        || add_short_cites(source_map),
    );

    debug!(slog_scope::logger(), "Source map complete.");
    source_map
}

/// Begin the source map, which contains only partial short cites
fn start_source_map<'a>(
    csl_library: Vec<CSLSource>,
    tree: &'a [Branch],
    user_journals: Option<UserJournals>,
) -> SourceMap<'a> {
    debug!(slog_scope::logger(), "Starting source map build...");

    // Create a blank source map.
    let mut source_map: SourceMap<'a> = HashMap::new();

    // Iterate through each branch in the tree, looking for footnotes.
    for branch in tree {
        if let Branch::Footnote(footnote) = branch {
            for footnote_branch in &footnote.contents {
                // Iterate through the footnote branches looking for citations.
                if let Branch::Citation(citation) = footnote_branch {
                    // Remove the brackets and @ to search through the CSL
                    // library's ids
                    let csl_id = &citation.reference[2..citation.reference.len() - 1];
                    if !source_map.contains_key(citation.reference) {
                        // Iterate through the CSL JSON library looking for the
                        // id
                        trace!(
                            slog_scope::logger(),
                            "Searching the CSL JSON library for {:?}...",
                            csl_id
                        );
                        let mut iter = csl_library.iter();
                        let csl_source = iter.find(|&x| x.id == csl_id);

                        // If found in the CSL JSON library, add it to the
                        // source map.
                        if let Some(csl_source) = csl_source {
                            trace!(
                                slog_scope::logger(),
                                "Found {:?} in the CSL JSON library.",
                                csl_id
                            );
                            // Determine the source type. If there's no type,
                            // don't add anything and log the problem. All
                            // supported source types should be in this match.
                            let source_type;
                            if csl_source.source_type.is_some() {
                                source_type =
                                    match csl_source.source_type.as_ref().unwrap().as_str() {
                                        "book" => SourceType::Book,
                                        "chapter" => SourceType::Chapter,
                                        "article-journal" => SourceType::JournalArticle,
                                        "manuscript" => SourceType::Manuscript,
                                        _ => SourceType::Other,
                                    };

                                if source_type != SourceType::Other {
                                    match slog_scope::scope(
                                        &slog_scope::logger().new(o!("fn" => "build_source()")),
                                        || {
                                            buildsource::build_source(
                                                csl_source,
                                                csl_id,
                                                source_type,
                                                &citation.pincite,
                                                footnote.number,
                                                &user_journals,
                                            )
                                        },
                                    ) {
                                        Ok(source) => {
                                            trace!(
                                                slog_scope::logger(),
                                                "Adding source {}",
                                                Color::Blue.paint(csl_id)
                                            );
                                            source_map.insert(citation.reference, source);
                                        }
                                        Err(e) => {
                                            warn!(
                                                slog_scope::logger(),
                                                "Cannoit build citation for {}; {}",
                                                Color::Blue.paint(csl_id),
                                                e
                                            )
                                        }
                                    }
                                } else {
                                    // Unsupported type
                                    warn!(
                                        slog_scope::logger(),
                                        "{}'s type ({}) is not supported; not adding to source map",
                                        Color::Blue.paint(csl_id),
                                        csl_source.source_type.as_ref().unwrap(),
                                    );
                                }
                            } else {
                                // No type in the CSL library
                                warn!(
                                    slog_scope::logger(),
                                    "{} does not have a type; not adding to source map",
                                    Color::Blue.paint(csl_id),
                                );
                            }
                        } else {
                            // Citation was not found in the CSL JSON library
                            warn!(slog_scope::logger(), "{}  was not found in the CSL JSON library; not adding to source map", Color::Blue.paint(csl_id)
                        );
                        }
                    } else {
                        // The key is already in the sourcemap. Add the footnote
                        // number to the list of footnotes.
                        source_map
                            .get_mut(&citation.reference)
                            .unwrap()
                            .all_footnotes
                            .push(footnote.number);

                        trace!(
                            slog_scope::logger(),
                            "Adding footnote {} to {}",
                            footnote.number,
                            Color::Blue.paint(csl_id)
                        );
                    }
                }
            }
        }
    }

    debug!(slog_scope::logger(), "Source map completed.");
    source_map
}

/// Check which sources need "hereinafter" short cites.
///
/// Each `Source` contains a boolean `hereinafter` value for whether sources
/// with that short author need a hereinafter cite.
///
/// The function iterates through the sources in the source map, comparing their
/// short cite form. At this point, the short cite string is only the authors,
/// *e.g.* "Smith", "Smith & Jones", or "Smith et al.". If a short cite is not
/// in the author list, it's added to the list. If the short cite is in the
/// list, the `hereinafter` value is changed to true.
fn check_hereinafters(mut source_map: SourceMap<'_>) -> SourceMap<'_> {
    debug!(slog_scope::logger(), "Checking \"hereinafter\" needs...");
    // This will require two trips through the source_map: one to determine who
    // needs hereinafters, and another to set the field.

    // Determine which short authors need hereinafters.
    // Crate an author_list hash map. The keys are short authors, the values are
    // collections of Pandoc-style citations (the keys from the source_map).
    let mut author_list: HashMap<String, Vec<String>> = HashMap::new();

    for (id, source) in source_map.iter() {
        // Get rid of any bolding in the author field for comparisons.
        let unformatted_author = source.short_cite.replace("**", "");

        if !author_list.contains_key(&unformatted_author) {
            // First encounter with a short author requires adding the author to
            // the list with an empty vector (the first source and all
            // subsequent sources will be added to the vector in the code that
            // follows this if statement).
            author_list.insert(unformatted_author.clone(), Vec::new());
        }
        // Add the source to the collection of sources for an author in the
        // author_list.
        author_list
            .get_mut(&unformatted_author)
            .unwrap()
            .push(id.to_string());
    }

    // Now set the hereinafter field to true for any author whose collection of
    // Pandoc-style citations has a length greater than 1.
    for (_, source) in source_map.iter_mut() {
        let unformatted_author = source.short_cite.replace("**", "");

        if author_list[&unformatted_author].len() > 1 {
            source.hereinafter = true;
        }
    }

    debug!(slog_scope::logger(), "\"Hereinafter\" needs checked.");
    source_map
}

/// Rebuild short cites for sources that need a hereinafter short form.
///
/// Takes the mutable source map and determines who needs a hereinafter. The
/// author map must be set to true (meaning that the same short author is used
/// for multiple sources) and the source is cited more than once. If there is
/// only one citation to a source, there is no need for a hereinafter.
fn add_short_cites(mut source_map: SourceMap) -> SourceMap {
    debug!(
        slog_scope::logger(),
        "Starting \"hereinafter\" additions..."
    );

    for (_, source) in source_map.iter_mut() {
        if source.hereinafter && source.all_footnotes.len() > 1 {
            source.long_cite.push_str(" [hereinafter ");
            source.long_cite.push_str(&source.short_cite);
            source.long_cite.push_str(", ");
            source.long_cite.push_str(&source.short_title);
            source.long_cite.push(']');

            source.short_cite.push_str(", ");
            source.short_cite.push_str(&source.short_title);
        }
        source.short_cite.push_str(", *supra* note ");
        source
            .short_cite
            .push_str(&source.first_footnote.to_string());
    }

    debug!(slog_scope::logger(), "\"Hereinafters\" added.");
    source_map
}

#[cfg(test)]
mod tests {
    use super::*;

    mod hereinafter_tests {
        use super::*;

        #[test]
        fn cited_once() {
            let mut source_map = HashMap::from([
                (
                    "[@jones1999]",
                    Source {
                        id: "jones1999".to_string(),
                        source_type: SourceType::JournalArticle,
                        first_footnote: 1,
                        all_footnotes: vec![1, 2, 3],
                        long_cite: "Jones 1999 Long Cite".to_string(),
                        short_cite: "Jones".to_string(),
                        short_title: "Jones 1999 Short Name".to_string(),
                        cited: false,
                        hereinafter: true,
                    },
                ),
                (
                    "[@jones2021]",
                    Source {
                        id: "jones2021".to_string(),
                        source_type: SourceType::JournalArticle,
                        first_footnote: 1,
                        all_footnotes: vec![1],
                        long_cite: "Jones 2021 Long Cite".to_string(),
                        short_cite: "Jones".to_string(),
                        short_title: "Jones 2021 Short Name".to_string(),
                        cited: false,
                        hereinafter: true,
                    },
                ),
            ]);

            source_map = add_short_cites(source_map);

            assert_eq!(
                &source_map["[@jones1999]"].long_cite,
                "Jones 1999 Long Cite [hereinafter Jones, Jones 1999 Short Name]"
            );
            assert_eq!(
                &source_map["[@jones1999]"].short_cite,
                "Jones, Jones 1999 Short Name, *supra* note 1"
            );
            assert_eq!(
                &source_map["[@jones2021]"].long_cite,
                "Jones 2021 Long Cite"
            );
            assert_eq!(
                &source_map["[@jones2021]"].short_cite,
                "Jones, *supra* note 1"
            );
        }
    }
}
