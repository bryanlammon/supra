//! This module contains the functions for creating sources.

mod citetools;
mod journalnames;
mod replacements;

use crate::{
    csljson::CSLSource, sourcemap::Source, sourcemap::SourceType, userjournals::UserJournals,
};
use ansi_term::Color;
use slog::{error, o, trace, warn};

/// Builds the long- and (partial) short-form citations.
///
/// This function takes a `CSLSource` as well as an optional first-use pincite
/// and outputs three strings:
///
/// * a long-form citation,
/// * a partial short-form citation (consisting of only the author), and
/// * a short title.
///
/// The long-form citation will replace the first citation in the document. It
/// accordingly includes the pincite if that first citation included a pincite.
/// The long-form citation includes all fields required by the Indigo Book.
///
/// The partial short-form citation and short title will be used for later
/// building the full short-form citation. The partial short-form consists of
/// only the authors. The short title is the short title to use if the same
/// author is used in the short cite for multiple sources (*e.g.*, two articles
/// by Jones). After determining whether the same author is used in different
/// short cites, the program should add an appropriate "hereinafter" to the long
/// cite (with the short author and short title) and finish the short cite to
/// include the short title. Regardless of whether a the hereinafter for is
/// used, the program will need to add the "*supra* note #" to the short-form
/// citation.
///
/// The partial short-form and short title are collected here to avoid having to
/// go back to the `CSLSource`. After the `Source`s have been built, the
/// collection of `CSLSource`s should never be needed again.
pub fn build_source(
    csl_source: &CSLSource,
    id: &str,
    source_type: SourceType,
    first_pin: &Option<&str>,
    first_footnote: i32,
    user_journals: &Option<UserJournals>,
) -> Result<Source, String> {
    trace!(slog_scope::logger(), "csl_source: {:?}\nid: {:?}\n source_type: {:?}\nfirst_pin: {:?}\nfirst_footnote: {:?}\nuser_journals: {:?}", csl_source, id, source_type, first_pin, first_footnote, user_journals);

    ////////////////////////////////////////////////////////////////////////////
    //
    // Long-Form Cite
    //
    ////////////////////////////////////////////////////////////////////////////
    // Start with a large capacity string to avoid repeatedly reallocating
    // memory.
    let mut long_cite = String::with_capacity(1024);

    ////////////////////////////////////////////////////////////////////////////
    // Leading Volume
    //
    // Add the <volume> at the beginning of a book that has one
    ////////////////////////////////////////////////////////////////////////////
    if source_type == SourceType::Book && csl_source.volume.is_some() {
        long_cite.push_str(csl_source.volume.as_ref().unwrap());
        long_cite.push(' ');
    }

    ////////////////////////////////////////////////////////////////////////////
    // Authors
    //
    // Add the authors. If it's a book, also bold the authors.
    ////////////////////////////////////////////////////////////////////////////
    if csl_source.author.is_some() {
        let mut author = slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "build_long_author")),
            || citetools::build_long_author(csl_source.author.as_ref().unwrap()),
        );

        if source_type == SourceType::Book {
            author = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                citetools::bold(&author)
            });
        }

        author.push_str(", ");

        long_cite.push_str(&author);
    }

    ////////////////////////////////////////////////////////////////////////////
    // Title
    //
    // Add the title. If it's a book, also bold it. If it's a chapter, journal,
    // or manuscript, "reverse italicize" it. If there's no title, give up and
    // return an error.
    ////////////////////////////////////////////////////////////////////////////
    if csl_source.title.is_some() {
        if source_type == SourceType::Book {
            let title = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                citetools::bold(csl_source.title.as_ref().unwrap())
            });
            long_cite.push_str(&title);
        } else if source_type == SourceType::Chapter
            || source_type == SourceType::JournalArticle
            || source_type == SourceType::Manuscript
        {
            let title = slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
                || citetools::reverse_italicize(csl_source.title.as_ref().unwrap()),
            );
            long_cite.push_str(&title);
        }

        ////////////////////////////////////////////////////////////////////////
        // In
        //
        // Add the "in" if it's a book chapter.
        ////////////////////////////////////////////////////////////////////////
        if source_type == SourceType::Chapter {
            long_cite.push_str(", *in* ");
        }

        ////////////////////////////////////////////////////////////////////////
        // Volume
        //
        // Add the volume to articles, chapters, and manuscripts.
        ////////////////////////////////////////////////////////////////////////
        if (source_type == SourceType::Chapter
            || source_type == SourceType::JournalArticle
            || source_type == SourceType::Manuscript)
            && csl_source.volume.is_some()
        {
            if source_type == SourceType::JournalArticle || source_type == SourceType::Manuscript {
                long_cite.push_str(", ");
            }
            long_cite.push_str(csl_source.volume.as_ref().unwrap());
            long_cite.push(' ');
        }

        ////////////////////////////////////////////////////////////////////////
        // Book/Journal Name
        //
        // If a chapter or article, add the book or journal name.
        ////////////////////////////////////////////////////////////////////////
        if (source_type == SourceType::Chapter
            || source_type == SourceType::JournalArticle
            || source_type == SourceType::Manuscript)
            && csl_source.container_title.is_some()
        {
            let container_title;
            if source_type == SourceType::Chapter {
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

            long_cite.push_str(&container_title);
        }

        ////////////////////////////////////////////////////////////////////////
        // Forthcoming
        //
        // If it's a manuscript with a note and a date, add the journal
        // forthcoming parenthetical.
        ////////////////////////////////////////////////////////////////////////
        if source_type == SourceType::Manuscript
            && csl_source.issued.is_some()
            && csl_source.issued.as_ref().unwrap().date_parts.is_some()
        {
            long_cite.push_str(" (forthcoming ");
            long_cite.push_str(
                &csl_source
                    .issued
                    .as_ref()
                    .unwrap()
                    .date_parts
                    .as_ref()
                    .unwrap()[0][0]
                    .to_string(),
            );
            long_cite.push(')');
        }

        ////////////////////////////////////////////////////////////////////////
        // Page
        //
        // Add the first page.
        ////////////////////////////////////////////////////////////////////////
        if (source_type == SourceType::Chapter || source_type == SourceType::JournalArticle)
            && csl_source.page.is_some()
        {
            long_cite.push(' ');
            long_cite.push_str(csl_source.page.as_ref().unwrap());
        }

        ////////////////////////////////////////////////////////////////////////
        // Pincite
        //
        // Add the pin cite.
        ////////////////////////////////////////////////////////////////////////
        if first_pin.is_some() {
            if source_type == SourceType::Book {
                long_cite.push(' ');
                long_cite.push_str(first_pin.as_ref().unwrap());
            } else if source_type == SourceType::Manuscript {
                long_cite.push_str(" (manuscript at ");
                long_cite.push_str(first_pin.as_ref().unwrap());
                long_cite.push(')');
            } else {
                long_cite.push_str(", ");
                long_cite.push_str(first_pin.as_ref().unwrap());
            }
        }

        ////////////////////////////////////////////////////////////////////////
        // Ending Parenthetical
        //
        // Add the ending parenethical with edition, editors, translators, and
        // year. Note, if an article's volume is four digits, no parenthetical
        // (which would be the year) will be added. This is to account for
        // volumized years.
        ////////////////////////////////////////////////////////////////////////
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
            long_cite.push_str(" (");

            ////////////////////////////////////////////////////////////////////
            // Edition
            //
            // Add the edition.
            ////////////////////////////////////////////////////////////////////
            if (source_type == SourceType::Book || source_type == SourceType::Chapter)
                && csl_source.edition.is_some()
            {
                long_cite.push_str(csl_source.edition.as_ref().unwrap());
                long_cite.push_str(" ed.");

                // If an editor or translator follows the edition, add a comma.
                // Otherwise add a space.
                if csl_source.editor.is_some() || csl_source.translator.is_some() {
                    long_cite.push_str(", ");
                } else {
                    long_cite.push(' ');
                }
            }

            ////////////////////////////////////////////////////////////////////
            // Editors
            //
            // Add the editors.
            ////////////////////////////////////////////////////////////////////
            if (source_type == SourceType::Book || source_type == SourceType::Chapter)
                && csl_source.editor.is_some()
            {
                let editors = slog_scope::scope(
                    &slog_scope::logger().new(o!("fn" => "build_long_author")),
                    || citetools::build_long_author(csl_source.editor.as_ref().unwrap()),
                );
                long_cite.push_str(&editors);
                if csl_source.editor.as_ref().unwrap().len() > 1 {
                    long_cite.push_str(" eds., ");
                } else {
                    long_cite.push_str(" ed., ");
                }
            }

            ////////////////////////////////////////////////////////////////////
            // Translators
            //
            // Add the translators.
            ////////////////////////////////////////////////////////////////////
            if (source_type == SourceType::Book || source_type == SourceType::Chapter)
                && csl_source.translator.is_some()
            {
                let translators = slog_scope::scope(
                    &slog_scope::logger().new(o!("fn" => "build_long_author")),
                    || citetools::build_long_author(csl_source.translator.as_ref().unwrap()),
                );
                long_cite.push_str(&translators);
                long_cite.push_str(" trans., ");
            }

            ////////////////////////////////////////////////////////////////////
            // Year
            //
            // Add the year.
            ////////////////////////////////////////////////////////////////////
            if csl_source.issued.as_ref().unwrap().date_parts.is_some() {
                long_cite.push_str(
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
            long_cite.push(')');
        }

        ////////////////////////////////////////////////////////////////////////
        // URL
        //
        // Add the url.
        ////////////////////////////////////////////////////////////////////////
        if source_type == SourceType::Manuscript && csl_source.url.is_some() {
            long_cite.push_str(", ");
            long_cite.push_str(csl_source.url.as_ref().unwrap());
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Short-Form Cite
        //
        // Start the short cite, which for now is only the short-form author.
        //
        ////////////////////////////////////////////////////////////////////////
        let mut short_cite = String::with_capacity(256);
        if csl_source.author.is_some() {
            short_cite = slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "build_short_author()")),
                || citetools::build_short_author(csl_source.author.as_ref().unwrap()),
            );
            if source_type == SourceType::Book {
                short_cite = citetools::bold(&short_cite);
            }
        }

        ////////////////////////////////////////////////////////////////////////
        //
        // Short Title
        //
        // Get the shortened title, or use the long one if there is no shortened
        // title
        //
        ////////////////////////////////////////////////////////////////////////
        let short_title;
        if csl_source.title_short.is_some() {
            if source_type == SourceType::Book {
                short_title = citetools::bold(csl_source.title_short.as_ref().unwrap());
            } else {
                short_title = slog_scope::scope(
                    &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
                    || citetools::reverse_italicize(csl_source.title_short.as_ref().unwrap()),
                )
            }
        } else {
            short_title = slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
                || citetools::reverse_italicize(csl_source.title.as_ref().unwrap()),
            );
            warn!(
                slog_scope::logger(),
                "No short title found for {}; using long title for possible hereinafters",
                Color::Blue.paint(&csl_source.id)
            );
        }

        let new_source = Source {
            id: id.to_string(),
            source_type,
            long_cite,
            short_cite,
            first_footnote,
            all_footnotes: vec![first_footnote],
            short_title,
            cited: false,
            hereinafter: false,
        };

        trace!(
            slog_scope::logger(),
            "Completed building source: {:?}",
            new_source
        );
        Ok(new_source)
    } else {
        error!(
            slog_scope::logger(),
            "{} does not have a title", &csl_source.id
        );
        Err(format!("{} does not have a title", &csl_source.id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod build_tests {
        use super::*;
        use crate::csljson;

        #[test]
        fn build_book() {
            let raw_json = r#"[{"id":"authorBookTitleTitle2021","author":[{"family":"Author","given":"Book","suffix":"Sr."},{"family":"Author","given":"Book","suffix":"Jr."}],"edition":"4th","issued":{"date-parts":[[2021,8]]},"title":"Book Title: A Title for the Dummy Book","title-short":"Book Title","type":"book"}]"#;
            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
            let first_pin = Some("12");

            let book = build_source(
                csl_source,
                "authorBookTitleTitle2021",
                SourceType::Book,
                &first_pin,
                1,
                &None,
            )
            .unwrap();

            assert_eq!(book.long_cite, "**Book Author, Sr. & Book Author, Jr.**, **Book Title: A Title for the Dummy Book** 12 (4th ed. 2021)");
            assert_eq!(book.short_cite, "**Author & Author**");
            assert_eq!(book.short_title, "**Book Title**");
        }

        #[test]
        fn build_chapter() {
            let raw_json = r#"[{"id":"authorBookSectionTitle2021","author":[{"family":"Author","given":"Book","suffix":"Sr."},{"family":"Author","given":"Book","suffix":"III"}],"container-title":"The Title of the Sectioned Book","edition":"5th","editor":[{"family":"Editor","given":"Book"}],"issued":{"date-parts":[[2021,8,1]]},"page":"12","title":"Book Section Title: The Section of a Book","translator":[{"family":"Translator","given":"Book"}],"type":"chapter","volume":"99"}]"#;
            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
            let first_pin = Some("12");

            let chapter = build_source(
                csl_source,
                "authorBookSectionTitle2021",
                SourceType::Chapter,
                &first_pin,
                1,
                &None,
            )
            .unwrap();

            assert_eq!(chapter.long_cite, "Book Author, Sr. & Book Author III, *Book Section Title: The Section of a Book*, *in* 99 **The Title of the Sectioned Book** 12, 12 (5th ed., Book Editor ed., Book Translator trans., 2021)");
            assert_eq!(chapter.short_cite, "Author & Author");
            assert_eq!(
                chapter.short_title,
                "*Book Section Title: The Section of a Book*"
            );
        }
        #[test]
        fn build_article() {
            let raw_json = r#"[{"id":"authorJournalArticleTitle2021","author":[{"family":"Author","given":"Article"},{"family":"Author","given":"Article","suffix":"Jr."},{"family":"Third","given":"Article Author"}],"container-title":"Journal of Journal Articles","issued":{"season":2,"date-parts":[[2021]]},"page":"1000","title":"Journal Article Title: A Journal Article","title-short":"Journal Article","type":"article-journal","volume":"99"}]"#;
            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
            let first_pin = Some("1012");

            let article = build_source(
                csl_source,
                "authorJournalArticleTitle2021",
                SourceType::JournalArticle,
                &first_pin,
                1,
                &None,
            )
            .unwrap();

            assert_eq!(article.long_cite, "Article Author, Article Author, Jr. & Article Author Third, *Journal Article Title: A Journal Article*, 99 **J. J. Articles** 1000, 1012 (2021)");
            assert_eq!(article.short_cite, "Author et al.");
            assert_eq!(article.short_title, "*Journal Article*");
        }

        #[test]
        fn build_manuscript() {
            let raw_json = r#"[{"id":"authorManuscriptTitleNot2020","author":[{"family":"Author","given":"Manuscript"}],"container-title":"University of Manuscripts Law Review","issued":{"date-parts":[[2020]]},"title":"Manuscript Title: Not Yet a Journal Article","title-short":"Manuscript Title","type":"manuscript","URL":"www.ssrn.com/manuscript", "volume":"45"}]"#;
            let csl_source = &csljson::build_csl_lib(raw_json).unwrap()[0];
            let first_pin = Some("12");

            let manuscript = build_source(
                csl_source,
                "authorManuscriptTitleNot2020",
                SourceType::Manuscript,
                &first_pin,
                1,
                &None,
            )
            .unwrap();

            assert_eq!(manuscript.long_cite, "Manuscript Author, *Manuscript Title: Not Yet a Journal Article*, 45 **U. Manuscripts L. Rev.** (forthcoming 2020) (manuscript at 12), www.ssrn.com/manuscript");
            assert_eq!(manuscript.short_cite, "Author");
            assert_eq!(manuscript.short_title, "*Manuscript Title*");
        }
    }
}
