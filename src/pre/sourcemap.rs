//! The module contains the functionality for creating the source map.

mod buildsource;

use super::csljson::CSLSource;
use super::parser::Branch;
use super::userjournals::UserJournals;
use ansi_term::Color;
use slog::{debug, o, trace, warn};
use std::collections::HashMap;

pub type SourceMap<'a> = HashMap<&'a str, Source<'a>>;

/// A source cited in the input document and built from the CSL JSON library.
///
/// The fields are used as follows:
///
/// * `csl_source`: A reference to the [`CSLSource`] data for the `Source`,
///   which is used for creating the long and short cites.
/// * `id`: The ID for the `Source`.
/// * `source_type`: The type of source, stored as a [`SourceType`] enum
///   variant.
/// * `all_footnotes`: A collection of all of the footnotes in which the source
///   is cited.
/// * `long_cite_no_pin`: A [`String`] of the source's long cite, with no
///   pincite.
/// * `long_cite_with_pin`: A tuple of [`String`]s that are used for long cites
///   with pincites.
/// * `short_author`: The author used for short cites. This is used do determine
///   the need for hereinafters when building the source map.
/// * `short_cite`: The short form of the citation. TODO This should be broken
///   out into the with and without pin varieties to facilitate cases.
/// * `cited`: Whether the `Source` has been cited. Used to determine whether a
///   short form can be used. TODO This could be changed to `last_cited` and
///   used for determining whether `Id.` can be used instead of a short cite.
/// * `hereinafter`: Whether the `Source` requires a hereinafter.
#[derive(Debug)]
pub struct Source<'a> {
    pub csl_source: &'a CSLSource,
    pub id: String,
    pub source_type: SourceType,
    pub all_footnotes: Vec<i32>,
    pub long_cite_no_pin: Option<String>,
    pub long_cite_w_pin: Option<(String, String)>,
    pub short_author: Option<String>,
    pub short_cite: Option<String>,
    pub cited: bool,
    pub hereinafter: bool,
}

impl Source<'_> {
    /// Output a long cite with no pin.
    pub fn long_cite_no_pin(&self) -> String {
        self.long_cite_no_pin.as_ref().unwrap().to_string()
    }

    /// Output a long cite with a pin.
    pub fn long_cite_w_pin(&self, pin: &str) -> String {
        let mut cite = self.long_cite_w_pin.as_ref().unwrap().0.to_owned();
        if self.source_type == SourceType::Book {
            cite.push(' ');
            cite.push_str(pin);
        } else if self.source_type == SourceType::Manuscript {
            cite.push_str(" (manuscript at ");
            cite.push_str(pin);
            cite.push(')');
        } else {
            cite.push_str(", ");
            cite.push_str(pin);
        }
        cite.push_str(&self.long_cite_w_pin.as_ref().unwrap().1);

        cite
    }

    /// Output a short cite.
    pub fn short_cite(&self) -> String {
        self.short_cite.as_ref().unwrap().to_string()
    }
}

/// The types of currently supported sources, plus the catchall `Other`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
/// the citations in the document.
///
/// Creating the [`SourceMap`] requires four steps. It first creates the basic
/// information for each source. But full cites---short or long---cannot be
/// determined until after all sources have been collected. So the function must
/// then determine which sources need "hereinafters." After that, it creates
/// appropriate long cites. Then short cites are created.
///
/// Note, the identifiers in the [`SourceMap`] map include the brackets and at
/// symbol, *e.g.*, `[@smith2021]`. But the identifers in [`CSLSource`]s don't,
/// *e.g.*, `smith2021`.
pub fn build_source_map<'a>(
    tree: &'a [Branch],
    csl_library: &'a [CSLSource],
    user_journals: &'a Option<UserJournals>,
) -> SourceMap<'a> {
    debug!(slog_scope::logger(), "Starting source map...");

    // Start the source map.
    let mut source_map = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "start_source_map()")),
        || start_source_map(csl_library, tree),
    );

    // Determine the need for hereinafters
    slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "check_hereinafters()")),
        || check_hereinafters(&mut source_map),
    );

    // Add the long cites for every source.
    slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "add_long_cites()")),
        || add_long_cites(&mut source_map, user_journals),
    );

    // Add the short cites for every source.
    slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "add_short_cites()")),
        || add_short_cites(&mut source_map),
    );

    debug!(slog_scope::logger(), "Source map complete.");
    source_map
}

/// Begin the source map, which contains only partial info.
fn start_source_map<'a>(csl_library: &'a [CSLSource], tree: &'a [Branch]) -> SourceMap<'a> {
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

                    // If the key is already in the sourcemap, Add the footnote
                    // number to the list of footnotes and continue to the next
                    // iteration of the loop.
                    if source_map.contains_key(citation.reference) {
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

                        continue;
                    }

                    // Iterate through the CSL JSON library looking for the id
                    trace!(
                        slog_scope::logger(),
                        "Searching the CSL JSON library for {:?}...",
                        csl_id
                    );
                    let mut iter = csl_library.iter();
                    let csl_source = iter.find(|&x| x.id == csl_id);

                    // If the citation is not found in the CSL JSON library, log
                    // the error and continue to the next iteration of the loop.
                    if csl_source.is_none() {
                        warn!(
                            slog_scope::logger(),
                            "{} was not found in the CSL JSON library; not adding to source map",
                            Color::Blue.paint(csl_id)
                        );
                        eprintln!("  {} {} was not found in the CSL JSON library; not adding to source map",
                                Color::Yellow.paint("WARN"),
                                Color::Blue.paint(csl_id)
                            );

                        continue;
                    }

                    // If found in the CSL JSON library, add it to the source
                    // map.
                    trace!(
                        slog_scope::logger(),
                        "Found {:?} in the CSL JSON library.",
                        csl_id
                    );
                    let csl_source = csl_source.unwrap();

                    // If there's no type in the CSL JSON library, log the error and continue to the next iteration of the loop.
                    if csl_source.source_type.is_none() {
                        warn!(
                            slog_scope::logger(),
                            "{} does not have a type; not adding to source map",
                            Color::Blue.paint(csl_id),
                        );
                        eprintln!(
                            "  {} {} does not have a type; not adding to source map",
                            Color::Yellow.paint("WARN"),
                            Color::Blue.paint(csl_id)
                        );

                        continue;
                    }

                    // Determine the source type. All supported source types
                    // should be in this match.
                    let source_type = match csl_source.source_type.as_ref().unwrap().as_str() {
                        "book" => SourceType::Book,
                        "chapter" => SourceType::Chapter,
                        "article-journal" => SourceType::JournalArticle,
                        "manuscript" => SourceType::Manuscript,
                        _ => SourceType::Other,
                    };

                    // Unsupported type
                    if source_type == SourceType::Other {
                        warn!(
                            slog_scope::logger(),
                            "{}'s type ({}) is not supported; not adding to source map",
                            Color::Blue.paint(csl_id),
                            csl_source.source_type.as_ref().unwrap(),
                        );
                        eprintln!(
                            "  {} {}'s type ({}) is not supported; not adding to source map",
                            Color::Yellow.paint("WARN"),
                            Color::Blue.paint(csl_id),
                            csl_source.source_type.as_ref().unwrap(),
                        );

                        continue;
                    }

                    // Build the Source.
                    match slog_scope::scope(
                        &slog_scope::logger().new(o!("fn" => "build_source()")),
                        || {
                            buildsource::build_source(
                                csl_source,
                                csl_id,
                                source_type,
                                footnote.number,
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
                                "Cannot build citation for {}; {}",
                                Color::Blue.paint(csl_id),
                                e
                            );
                            eprintln!(
                                "  {} Cannot build citation for {}; {:?}",
                                Color::Yellow.paint("WARN"),
                                Color::Blue.paint(csl_id),
                                e
                            )
                        }
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
//fn check_hereinafters(mut source_map: SourceMap<'_>) -> SourceMap<'_> {
fn check_hereinafters(source_map: &mut SourceMap<'_>) {
    debug!(slog_scope::logger(), "Checking \"hereinafter\" needs...");
    // This will require two trips through the source_map: one to determine who
    // needs hereinafters, and another to set the field.

    // Determine which short authors need hereinafters. Crate an author_list
    // hash map. The keys are short authors, the values are collections of
    // Pandoc-style citations (the keys from the source_map).
    let mut author_list: HashMap<String, Vec<String>> = HashMap::new();

    for (id, source) in source_map.iter() {
        // Get rid of any bolding in the author field for comparisons.
        let unformatted_author = source.short_author.as_ref().unwrap().replace("**", "");

        if !author_list.contains_key(&unformatted_author) {
            // First encounter with a short author requires adding the author to
            // the list with an empty vector (the first source and all
            // subsequent sources will be added to the vector in the code that
            // follows this if statement).
            author_list.insert(unformatted_author.to_owned(), Vec::new());
        }
        // Add the source to the collection of sources for an author in the
        // author_list.
        author_list
            .get_mut(&unformatted_author)
            .unwrap()
            .push(id.to_string());
    }

    // Now set the hereinafter field to true for any author whose collection of
    // citations has a length greater than 1.
    for (_, source) in source_map.iter_mut() {
        let unformatted_author = source.short_author.as_ref().unwrap().replace("**", "");

        if author_list[&unformatted_author].len() > 1 {
            source.hereinafter = true;
        }
    }

    debug!(slog_scope::logger(), "\"Hereinafter\" needs checked.");
}

/// Add long cites to sources.
fn add_long_cites<'a>(source_map: &mut SourceMap<'a>, user_journals: &'a Option<UserJournals>) {
    debug!(slog_scope::logger(), "Adding long cites...");

    for (_, source) in source_map.iter_mut() {
        let (long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin) =
            buildsource::build_long_cite(
                source.csl_source,
                source.source_type,
                user_journals,
                source.hereinafter,
            )
            .unwrap();

        source.long_cite_no_pin = Some(long_cite_no_pin);
        source.long_cite_w_pin = Some((long_cite_pre_pin, long_cite_post_pin));
    }

    debug!(slog_scope::logger(), "Long cites added.");
}

/// Add short cites to sources.
fn add_short_cites(source_map: &mut SourceMap<'_>) {
    debug!(slog_scope::logger(), "Adding short cites...");

    for (_, source) in source_map.iter_mut() {
        source.short_cite = Some(buildsource::build_short_cite(
            source.csl_source,
            &source.source_type,
            source.all_footnotes[0],
            source.hereinafter,
        ));
    }

    debug!(slog_scope::logger(), "Short cites added.");
}
