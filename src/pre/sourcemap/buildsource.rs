//! This module contains the functions for creating sources.

mod citetools;
mod journalnames;
mod replacements;

use crate::pre::{
    csljson::CSLSource, sourcemap::Source, sourcemap::SourceType, userjournals::UserJournals,
};
use ansi_term::Color;
use slog::{error, o, trace, warn};

/// Builds a `Source`.
pub fn build_source<'a>(
    csl_source: &'a CSLSource,
    id: &'a str,
    source_type: SourceType,
    //first_pin: &'a Option<&'a str>,
    first_footnote: i32,
    //user_journals: &'a Option<UserJournals>,
) -> Result<Source<'a>, String> {
    trace!(
        slog_scope::logger(),
        "building source---csl_source: {:?}",
        csl_source
    );

    // A short author is necessary to check hereinafters.
    let mut short_author = String::new();
    add_short_author(csl_source, &source_type, &mut short_author);

    let new_source = Source {
        csl_source,
        id: id.to_string(),
        source_type,
        first_footnote,
        all_footnotes: vec![first_footnote],
        long_cite_no_pin: None,
        long_cite_w_pin: None,
        short_cite: None,
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

    // Start with a large capacity string to avoid repeatedly reallocating
    // memory. We're also starting with the long cite before any pincites. We'll combine this with the post-pincite portion later to creat a full, no-pincite long cite.
    let mut long_cite_pre_pin = String::with_capacity(1024);

    if source_type == SourceType::Book && csl_source.volume.is_some() {
        add_book_volume(csl_source, &mut long_cite_pre_pin);
    }

    if csl_source.author.is_some() {
        add_authors(csl_source, &source_type, &mut long_cite_pre_pin);
    }

    if csl_source.title.is_some() {
        add_title(csl_source, &source_type, &mut long_cite_pre_pin);
    }

    if source_type == SourceType::Chapter {
        add_in(&mut long_cite_pre_pin);
    }

    if (source_type == SourceType::Chapter
        || source_type == SourceType::JournalArticle
        || source_type == SourceType::Manuscript)
        && csl_source.volume.is_some()
    {
        add_other_volume(csl_source, &source_type, &mut long_cite_pre_pin);
    }

    if (source_type == SourceType::Chapter
        || source_type == SourceType::JournalArticle
        || source_type == SourceType::Manuscript)
        && csl_source.container_title.is_some()
    {
        add_book_name(
            csl_source,
            &source_type,
            user_journals,
            &mut long_cite_pre_pin,
        );
    }

    if source_type == SourceType::Manuscript
        && csl_source.issued.is_some()
        && csl_source.issued.as_ref().unwrap().date_parts.is_some()
    {
        add_forthcoming(csl_source, &mut long_cite_pre_pin);
    }

    if (source_type == SourceType::Chapter || source_type == SourceType::JournalArticle)
        && csl_source.page.is_some()
    {
        add_first_page(csl_source, &mut long_cite_pre_pin);
    }

    // Create another empty string for the post-pincite portion.
    let mut long_cite_post_pin = String::with_capacity(1024);

    // Note, if an article's volume is four digits, no parenthetical (which
    // would be the year) will be added. This is to account for volumized years.
    if (source_type == SourceType::Book
        || source_type == SourceType::Chapter
        || (source_type == SourceType::JournalArticle
            && csl_source.volume.is_some()
            && csl_source.volume.as_ref().unwrap().len() != 4))
        && (csl_source.edition.is_some()
            || csl_source.editor.is_some()
            || csl_source.translator.is_some()
            || csl_source.issued.is_some())
    {
        add_end_parenthetical(csl_source, &source_type, &mut long_cite_post_pin);
    }

    // Add a hereinafter, if necessary
    if hereinafter {
        add_hereinafter(csl_source, &source_type, &mut long_cite_post_pin);
    }

    if source_type == SourceType::Manuscript && csl_source.url.is_some() {
        add_url(csl_source, &mut long_cite_post_pin);
    }

    let mut long_cite_no_pin = long_cite_pre_pin.clone();
    long_cite_no_pin.push_str(&long_cite_post_pin);

    trace!(
        slog_scope::logger(),
        "Long cite built: {}",
        long_cite_no_pin
    );
    Ok((long_cite_no_pin, long_cite_pre_pin, long_cite_post_pin))
}

/// Build the short cite.
pub fn build_short_cite(
    csl_source: &CSLSource,
    source_type: &SourceType,
    first_footnote: i32,
    hereinafter: bool,
) -> String {
    let mut cite = String::with_capacity(256);

    if csl_source.author.is_some() {
        add_short_author(csl_source, source_type, &mut cite);
    }

    if hereinafter {
        cite.push_str(", ");
        add_short_title(csl_source, source_type, &mut cite);
    }

    cite.push_str(", *supra* note ");
    cite.push_str(&first_footnote.to_string());

    cite
}

/// Add the volume at the beginning of a book that has one.
fn add_book_volume(csl_source: &CSLSource, cite: &mut String) {
    cite.push_str(csl_source.volume.as_ref().unwrap());
    cite.push(' ');
}

/// Add the authors. If it's a book, also bold the authors.
fn add_authors(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    let mut author = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_long_author")),
        || citetools::build_long_author(csl_source.author.as_ref().unwrap()),
    );

    if source_type == &SourceType::Book {
        author = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
            citetools::bold(&author)
        });
    }

    author.push_str(", ");

    cite.push_str(&author);
}

/// Add the short author for hereinafters and short cites.
fn add_short_author(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    let mut short_author = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_short_author()")),
        || citetools::build_short_author(csl_source.author.as_ref().unwrap()),
    );
    if source_type == &SourceType::Book {
        short_author = citetools::bold(&short_author);
    }

    cite.push_str(&short_author);
}

/// Add the title.
///
/// If it's a book, also bold it. If it's a chapter, journal, or manuscript,
/// "reverse italicize" it.
fn add_title(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    if source_type == &SourceType::Book {
        let title = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
            citetools::bold(csl_source.title.as_ref().unwrap())
        });
        cite.push_str(&title);
    } else if source_type == &SourceType::Chapter
        || source_type == &SourceType::JournalArticle
        || source_type == &SourceType::Manuscript
    {
        let title = slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
            || citetools::reverse_italicize(csl_source.title.as_ref().unwrap()),
        );
        cite.push_str(&title);
    }
}

/// Add the shortened title (if it exists).
///
/// This adds the shortened title for a source. If there isn't one, it uses the long title and warns the user.
fn add_short_title(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    let short_title;
    if csl_source.title_short.is_some() {
        if source_type == &SourceType::Book {
            short_title = citetools::bold(csl_source.title_short.as_ref().unwrap());
        } else {
            short_title = slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
                || citetools::reverse_italicize(csl_source.title_short.as_ref().unwrap()),
            )
        }
    } else {
        short_title = if source_type == &SourceType::Book {
            citetools::bold(csl_source.title.as_ref().unwrap())
        } else {
            slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
                || citetools::reverse_italicize(csl_source.title.as_ref().unwrap()),
            )
        };
        warn!(
            slog_scope::logger(),
            "No short title found for {}; using long title for possible hereinafters",
            Color::Blue.paint(&csl_source.id)
        );
        eprintln!(
            "  {} No short title found for {}; using long title for possible hereinafters",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(&csl_source.id)
        )
    }

    cite.push_str(&short_title);
}

/// Add the "in" if it's a book chapter.
fn add_in(cite: &mut String) {
    cite.push_str(", *in* ");
}

/// Add the volume to articles, chapters, and manuscripts.
fn add_other_volume(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    if source_type == &SourceType::JournalArticle || source_type == &SourceType::Manuscript {
        cite.push_str(", ");
    }
    cite.push_str(csl_source.volume.as_ref().unwrap());
    cite.push(' ');
}

/// The book or journal name to a chapter or article.
fn add_book_name(
    csl_source: &CSLSource,
    source_type: &SourceType,
    user_journals: &Option<UserJournals>,
    cite: &mut String,
) {
    let container_title;
    if source_type == &SourceType::Chapter {
        container_title =
            slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                citetools::bold(csl_source.container_title.as_ref().unwrap())
            });
    } else if csl_source.container_title_short.is_some() {
        container_title =
            slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                citetools::bold(csl_source.container_title_short.as_ref().unwrap())
            });
    } else {
        let short_journal = slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "build_short_journal()")),
            || {
                citetools::build_short_journal(
                    csl_source.container_title.as_ref().unwrap(),
                    user_journals,
                )
            },
        );
        container_title =
            slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                citetools::bold(&short_journal)
            });
    }

    cite.push_str(&container_title);
}

/// Add the journal-forthcoming parenthetical.
///
/// Used for manuscripts with a note and a date.
fn add_forthcoming(csl_source: &CSLSource, cite: &mut String) {
    cite.push_str(" (forthcoming ");
    cite.push_str(
        &csl_source
            .issued
            .as_ref()
            .unwrap()
            .date_parts
            .as_ref()
            .unwrap()[0][0]
            .to_string(),
    );
    cite.push(')');
}

/// Add the first page.
fn add_first_page(csl_source: &CSLSource, cite: &mut String) {
    cite.push(' ');
    cite.push_str(csl_source.page.as_ref().unwrap());
}

/// Add the ending parenthetical with edition, editors, translators, and year.
fn add_end_parenthetical(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    cite.push_str(" (");

    // Add the edition.
    if (source_type == &SourceType::Book || source_type == &SourceType::Chapter)
        && csl_source.edition.is_some()
    {
        cite.push_str(csl_source.edition.as_ref().unwrap());
        cite.push_str(" ed.");

        // If an editor or translator follows the edition, add a comma.
        // Otherwise add a space.
        if csl_source.editor.is_some() || csl_source.translator.is_some() {
            cite.push_str(", ");
        } else {
            cite.push(' ');
        }
    }

    // Add the editors.
    if (source_type == &SourceType::Book || source_type == &SourceType::Chapter)
        && csl_source.editor.is_some()
    {
        let editors = slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "build_long_author")),
            || citetools::build_long_author(csl_source.editor.as_ref().unwrap()),
        );
        cite.push_str(&editors);
        if csl_source.editor.as_ref().unwrap().len() > 1 {
            cite.push_str(" eds., ");
        } else {
            cite.push_str(" ed., ");
        }
    }

    // Add the translators.
    if (source_type == &SourceType::Book || source_type == &SourceType::Chapter)
        && csl_source.translator.is_some()
    {
        let translators = slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "build_long_author")),
            || citetools::build_long_author(csl_source.translator.as_ref().unwrap()),
        );
        cite.push_str(&translators);
        cite.push_str(" trans., ");
    }

    // Add the year.
    if csl_source.issued.as_ref().unwrap().date_parts.is_some() {
        cite.push_str(
            &csl_source
                .issued
                .as_ref()
                .unwrap()
                .date_parts
                .as_ref()
                .unwrap()[0][0]
                .to_string(),
        );
    }

    // Close the parentheses
    cite.push(')');
}

/// Add the "hereinafter" to sources that need it.
fn add_hereinafter(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    cite.push_str(" [hereinafter ");
    add_short_author(csl_source, source_type, cite);
    cite.push_str(", ");
    add_short_title(csl_source, source_type, cite);
    cite.push(']');
}

/// Add the url.
fn add_url(csl_source: &CSLSource, cite: &mut String) {
    cite.push_str(", ");
    cite.push_str(csl_source.url.as_ref().unwrap());
}

//#[cfg(test)]
//mod tests {
//    use super::*;

//    mod build_tests {
//        use super::*;
//        use crate::pre::csljson;

//        #[test]
//        fn build_book() {
//            let raw_json = r#"[{"id":"authorBookTitleTitle2021","author":[{"family":"Author","given":"Book","suffix":"Sr."},{"family":"Author","given":"Book","suffix":"Jr."}],"edition":"4th","issued":{"date-parts":[[2021,8]]},"title":"Book Title: A Title for the Dummy Book","title-short":"Book Title","type":"book"}]"#;
//            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
//            let first_pin = Some("12");

//            let book = build_source(
//                csl_source,
//                "authorBookTitleTitle2021",
//                SourceType::Book,
//                &first_pin,
//                1,
//                &None,
//            )
//            .unwrap();

//            assert_eq!(book.long_cite, "**Book Author, Sr. & Book Author, Jr.**, **Book Title: A Title for the Dummy Book** 12 (4th ed. 2021)");
//            assert_eq!(book.short_cite, "**Author & Author**");
//            assert_eq!(book.short_title, "**Book Title**");
//        }

//        #[test]
//        fn build_chapter() {
//            let raw_json = r#"[{"id":"authorBookSectionTitle2021","author":[{"family":"Author","given":"Book","suffix":"Sr."},{"family":"Author","given":"Book","suffix":"III"}],"container-title":"The Title of the Sectioned Book","edition":"5th","editor":[{"family":"Editor","given":"Book"}],"issued":{"date-parts":[[2021,8,1]]},"page":"12","title":"Book Section Title: The Section of a Book","translator":[{"family":"Translator","given":"Book"}],"type":"chapter","volume":"99"}]"#;
//            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
//            let first_pin = Some("12");

//            let chapter = build_source(
//                csl_source,
//                "authorBookSectionTitle2021",
//                SourceType::Chapter,
//                &first_pin,
//                1,
//                &None,
//            )
//            .unwrap();

//            assert_eq!(chapter.long_cite, "Book Author, Sr. & Book Author III, *Book Section Title: The Section of a Book*, *in* 99 **The Title of the Sectioned Book** 12, 12 (5th ed., Book Editor ed., Book Translator trans., 2021)");
//            assert_eq!(chapter.short_cite, "Author & Author");
//            assert_eq!(
//                chapter.short_title,
//                "*Book Section Title: The Section of a Book*"
//            );
//        }
//        #[test]
//        fn build_article() {
//            let raw_json = r#"[{"id":"authorJournalArticleTitle2021","author":[{"family":"Author","given":"Article"},{"family":"Author","given":"Article","suffix":"Jr."},{"family":"Third","given":"Article Author"}],"container-title":"Journal of Journal Articles","issued":{"season":2,"date-parts":[[2021]]},"page":"1000","title":"Journal Article Title: A Journal Article","title-short":"Journal Article","type":"article-journal","volume":"99"}]"#;
//            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
//            let first_pin = Some("1012");

//            let article = build_source(
//                csl_source,
//                "authorJournalArticleTitle2021",
//                SourceType::JournalArticle,
//                &first_pin,
//                1,
//                &None,
//            )
//            .unwrap();

//            assert_eq!(article.long_cite, "Article Author, Article Author, Jr. & Article Author Third, *Journal Article Title: A Journal Article*, 99 **J. J. Articles** 1000, 1012 (2021)");
//            assert_eq!(article.short_cite, "Author et al.");
//            assert_eq!(article.short_title, "*Journal Article*");
//        }

//        #[test]
//        fn build_manuscript() {
//            let raw_json = r#"[{"id":"authorManuscriptTitleNot2020","author":[{"family":"Author","given":"Manuscript"}],"container-title":"University of Manuscripts Law Review","issued":{"date-parts":[[2020]]},"title":"Manuscript Title: Not Yet a Journal Article","title-short":"Manuscript Title","type":"manuscript","URL":"www.ssrn.com/manuscript", "volume":"45"}]"#;
//            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
//            let first_pin = Some("12");

//            let manuscript = build_source(
//                csl_source,
//                "authorManuscriptTitleNot2020",
//                SourceType::Manuscript,
//                &first_pin,
//                1,
//                &None,
//            )
//            .unwrap();

//            assert_eq!(manuscript.long_cite, "Manuscript Author, *Manuscript Title: Not Yet a Journal Article*, 45 **U. Manuscripts L. Rev.** (forthcoming 2020) (manuscript at 12), www.ssrn.com/manuscript");
//            assert_eq!(manuscript.short_cite, "Author");
//            assert_eq!(manuscript.short_title, "*Manuscript Title*");
//        }
//    }
//}
