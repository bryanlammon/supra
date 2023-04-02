//! This module contains the functions for creating sources.

mod build_types;
mod citetools;
mod journalnames;
mod replacements;

use crate::pre::{
    csljson::CSLSource, sourcemap::Source, sourcemap::SourceType, userjournals::UserJournals,
};
use slog::{error, trace};

/// Builds a `Source`.
pub fn build_source<'a>(
    csl_source: &'a CSLSource,
    id: &'a str,
    source_type: SourceType,
    first_footnote: i32,
) -> Result<Source<'a>, String> {
    trace!(
        slog_scope::logger(),
        "building source---csl_source: {:?}",
        csl_source
    );

    // A short author is necessary to check hereinafters.
    let mut short_author = String::new();
    if source_type != SourceType::Case {
        citetools::add_short_author(csl_source, &source_type, &mut short_author);
    }

    let all_footnotes = vec![first_footnote];

    let new_source = Source {
        csl_source,
        id: id.to_string(),
        source_type,
        all_footnotes,
        long_cite_no_pin: None,
        long_cite_w_pin: None,
        short_cite_no_pin: None,
        short_cite_w_pin: None,
        short_author: Some(short_author),
        cited: false,
        hereinafter: false,
    };

    trace!(
        slog_scope::logger(),
        "Completed building source: {:?}",
        new_source
    );
    Ok(new_source)
}

/// Build the long cite---with and without a pin---for a source.
pub fn build_long_cite(
    csl_source: &CSLSource,
    source_type: SourceType,
    user_journals: &Option<UserJournals>,
    hereinafter: bool,
) -> Result<(String, String, String), String> {
    trace!(
        slog_scope::logger(),
        "creating long cite---csl_source: {:?}",
        csl_source
    );

    // If there's no title, give up and return an error.
    if csl_source.title.is_none() {
        error!(
            slog_scope::logger(),
            "{} does not have a title", &csl_source.id
        );

        return Err(format!("{} does not have a title", &csl_source.id));
    }

    let (long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin) = match source_type {
        SourceType::Book => build_types::build_book_long(csl_source, &source_type, hereinafter),
        SourceType::Case => build_types::build_case_long(csl_source, &source_type, user_journals),
        SourceType::Chapter => {
            build_types::build_chapter_long(csl_source, &source_type, user_journals, hereinafter)
        }
        SourceType::JournalArticle => build_types::build_journal_article_long(
            csl_source,
            &source_type,
            user_journals,
            hereinafter,
        ),
        SourceType::Manuscript => {
            build_types::build_manuscript_long(csl_source, &source_type, user_journals, hereinafter)
        }
        SourceType::Other => {
            return Err("Cannot build cite for source type `Other`".to_string());
        }
    };

    trace!(
        slog_scope::logger(),
        "Long cite built: {}",
        long_cite_no_pin
    );
    Ok((long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin))
}

/// Build the short cite---with and without a pin--for a source.
pub fn build_short_cite(
    csl_source: &CSLSource,
    source_type: &SourceType,
    first_footnote: i32,
    hereinafter: bool,
) -> (String, String) {
    let mut short_cite = String::with_capacity(256);

    if source_type == &SourceType::Case {
        citetools::add_short_title(csl_source, source_type, &mut short_cite);
        if csl_source.volume.is_some() {
            citetools::add_other_volume(csl_source, source_type, &mut short_cite);
        }

        if csl_source.container_title.is_some() {
            short_cite.push_str(&csl_source.container_title.as_ref().unwrap().to_string());
        }

        if csl_source.page.is_some() {
            let short_cite_no_pin = format!("{} {}", short_cite, csl_source.page.as_ref().unwrap());
            (short_cite_no_pin, short_cite)
        } else {
            (short_cite.clone(), short_cite)
        }
    } else {
        if csl_source.author.is_some() {
            citetools::add_short_author(csl_source, source_type, &mut short_cite);
        }

        if hereinafter {
            short_cite.push_str(", ");
            citetools::add_short_title(csl_source, source_type, &mut short_cite);
        }

        short_cite.push_str(", *supra* note ");
        short_cite.push_str(&first_footnote.to_string());

        (short_cite.clone(), short_cite)
    }
}
