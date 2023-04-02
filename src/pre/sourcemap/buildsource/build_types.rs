//! Functions for building different kinds of sources.

use super::citetools;
use crate::pre::{csljson::CSLSource, sourcemap::SourceType, userjournals::UserJournals};

/// Build the long cite---with and without a pin---for a book.
pub fn build_book_long(
    csl_source: &CSLSource,
    source_type: &SourceType,
    //user_journals: &Option<UserJournals>,
    hereinafter: bool,
) -> (String, String, String) {
    // Start with a large capacity string to avoid repeatedly reallocating
    // memory. We're also starting with the long cite before any pincites. We'll
    // combine this with the post-pincite portion later to creat a full,
    // no-pincite long cite.
    let mut long_cite_pre_pin = String::with_capacity(1024);

    if csl_source.volume.is_some() {
        citetools::add_book_volume(csl_source, &mut long_cite_pre_pin);
    };

    if csl_source.author.is_some() {
        citetools::add_authors(csl_source, source_type, &mut long_cite_pre_pin);
    }

    citetools::add_title(csl_source, source_type, &mut long_cite_pre_pin);

    // Create another empty string for the post-pincite portion.
    let mut long_cite_post_pin = String::with_capacity(1024);

    if csl_source.edition.is_some()
        || csl_source.editor.is_some()
        || csl_source.translator.is_some()
        || csl_source.issued.is_some()
    {
        citetools::add_end_parenthetical(csl_source, source_type, &mut long_cite_post_pin);
    }

    if hereinafter {
        citetools::add_hereinafter(csl_source, source_type, &mut long_cite_post_pin);
    }

    let mut long_cite_no_pin = long_cite_pre_pin.clone();
    long_cite_no_pin.push_str(&long_cite_post_pin);

    (long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin)
}

/// Build the long cite---with and without a pin---for a case.
pub fn build_case_long(
    csl_source: &CSLSource,
    source_type: &SourceType,
    user_journals: &Option<UserJournals>,
) -> (String, String, String) {
    // Start with a large capacity string to avoid repeatedly reallocating
    // memory. We're also starting with the long cite before any pincites. We'll
    // combine this with the post-pincite portion later to creat a full,
    // no-pincite long cite.
    let mut long_cite_pre_pin = String::with_capacity(1024);

    citetools::add_title(csl_source, source_type, &mut long_cite_pre_pin);

    if csl_source.volume.is_some() {
        citetools::add_other_volume(csl_source, source_type, &mut long_cite_pre_pin);
    }

    if csl_source.container_title.is_some() {
        citetools::add_container_name(
            csl_source,
            source_type,
            user_journals,
            &mut long_cite_pre_pin,
        );
    }

    if csl_source.page.is_some() {
        citetools::add_first_page(csl_source, &mut long_cite_pre_pin);
    }

    // Create another empty string for the post-pincite portion.
    let mut long_cite_post_pin = String::with_capacity(1024);

    if csl_source.authority.is_some() || csl_source.issued.is_some() {
        citetools::add_end_parenthetical(csl_source, source_type, &mut long_cite_post_pin);
    }

    let mut long_cite_no_pin = long_cite_pre_pin.clone();
    long_cite_no_pin.push_str(&long_cite_post_pin);

    (long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin)
}

/// Build the long cite---with and without a pin---for a chapter.
pub fn build_chapter_long(
    csl_source: &CSLSource,
    source_type: &SourceType,
    user_journals: &Option<UserJournals>,
    hereinafter: bool,
) -> (String, String, String) {
    // Start with a large capacity string to avoid repeatedly reallocating
    // memory. We're also starting with the long cite before any pincites. We'll
    // combine this with the post-pincite portion later to creat a full,
    // no-pincite long cite.
    let mut long_cite_pre_pin = String::with_capacity(1024);

    if csl_source.author.is_some() {
        citetools::add_authors(csl_source, source_type, &mut long_cite_pre_pin);
    }

    citetools::add_title(csl_source, source_type, &mut long_cite_pre_pin);

    citetools::add_in(&mut long_cite_pre_pin);

    if csl_source.volume.is_some() {
        citetools::add_other_volume(csl_source, source_type, &mut long_cite_pre_pin);
    }

    if csl_source.container_title.is_some() {
        citetools::add_container_name(
            csl_source,
            source_type,
            user_journals,
            &mut long_cite_pre_pin,
        );
    }

    if csl_source.page.is_some() {
        citetools::add_first_page(csl_source, &mut long_cite_pre_pin);
    }

    // Create another empty string for the post-pincite portion.
    let mut long_cite_post_pin = String::with_capacity(1024);

    if csl_source.edition.is_some()
        || csl_source.editor.is_some()
        || csl_source.translator.is_some()
        || csl_source.issued.is_some()
    {
        citetools::add_end_parenthetical(csl_source, source_type, &mut long_cite_post_pin);
    }

    if hereinafter {
        citetools::add_hereinafter(csl_source, source_type, &mut long_cite_post_pin);
    }

    let mut long_cite_no_pin = long_cite_pre_pin.clone();
    long_cite_no_pin.push_str(&long_cite_post_pin);

    (long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin)
}

/// Build the long cite---with and without a pin---for a journal article.
pub fn build_journal_article_long(
    csl_source: &CSLSource,
    source_type: &SourceType,
    user_journals: &Option<UserJournals>,
    hereinafter: bool,
) -> (String, String, String) {
    // Start with a large capacity string to avoid repeatedly reallocating
    // memory. We're also starting with the long cite before any pincites. We'll
    // combine this with the post-pincite portion later to creat a full,
    // no-pincite long cite.
    let mut long_cite_pre_pin = String::with_capacity(1024);

    if csl_source.author.is_some() {
        citetools::add_authors(csl_source, source_type, &mut long_cite_pre_pin);
    }

    citetools::add_title(csl_source, source_type, &mut long_cite_pre_pin);

    if csl_source.volume.is_some() {
        citetools::add_other_volume(csl_source, source_type, &mut long_cite_pre_pin);
    }

    if csl_source.container_title.is_some() {
        citetools::add_container_name(
            csl_source,
            source_type,
            user_journals,
            &mut long_cite_pre_pin,
        );
    }

    if csl_source.page.is_some() {
        citetools::add_first_page(csl_source, &mut long_cite_pre_pin);
    }

    // Create another empty string for the post-pincite portion.
    let mut long_cite_post_pin = String::with_capacity(1024);

    if (csl_source.volume.is_some() && csl_source.volume.as_ref().unwrap().len() != 4)
        && (csl_source.edition.is_some()
            || csl_source.editor.is_some()
            || csl_source.translator.is_some()
            || csl_source.issued.is_some())
    {
        citetools::add_end_parenthetical(csl_source, source_type, &mut long_cite_post_pin);
    }

    if hereinafter {
        citetools::add_hereinafter(csl_source, source_type, &mut long_cite_post_pin);
    }

    let mut long_cite_no_pin = long_cite_pre_pin.clone();
    long_cite_no_pin.push_str(&long_cite_post_pin);

    (long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin)
}

/// Build the long cite---with and without a pin---for a manuscript.
pub fn build_manuscript_long(
    csl_source: &CSLSource,
    source_type: &SourceType,
    user_journals: &Option<UserJournals>,
    hereinafter: bool,
) -> (String, String, String) {
    // Start with a large capacity string to avoid repeatedly reallocating
    // memory. We're also starting with the long cite before any pincites. We'll
    // combine this with the post-pincite portion later to creat a full,
    // no-pincite long cite.
    let mut long_cite_pre_pin = String::with_capacity(1024);

    if csl_source.author.is_some() {
        citetools::add_authors(csl_source, source_type, &mut long_cite_pre_pin);
    }

    citetools::add_title(csl_source, source_type, &mut long_cite_pre_pin);

    if csl_source.volume.is_some() {
        citetools::add_other_volume(csl_source, source_type, &mut long_cite_pre_pin);
    }

    if csl_source.container_title.is_some() {
        citetools::add_container_name(
            csl_source,
            source_type,
            user_journals,
            &mut long_cite_pre_pin,
        );
    }

    if csl_source.issued.is_some() && csl_source.issued.as_ref().unwrap().date_parts.is_some() {
        citetools::add_forthcoming(csl_source, &mut long_cite_pre_pin);
    }

    // Create another empty string for the post-pincite portion.
    let mut long_cite_post_pin = String::with_capacity(1024);

    if hereinafter {
        citetools::add_hereinafter(csl_source, source_type, &mut long_cite_post_pin);
    }

    if csl_source.url.is_some() {
        citetools::add_url(csl_source, &mut long_cite_post_pin);
    }

    let mut long_cite_no_pin = long_cite_pre_pin.clone();
    long_cite_no_pin.push_str(&long_cite_post_pin);

    (long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin)
}
