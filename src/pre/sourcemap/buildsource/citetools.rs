//! This module contains tools for creating citations.

use super::{journalnames, replacements};
use crate::pre::{
    csljson::{CSLSource, NameVariable},
    sourcemap::SourceType,
    userjournals::UserJournals,
};
use ansi_term::Color;
use lazy_static::lazy_static;
use regex::Regex;
use slog::{o, trace, warn};

lazy_static! {
    /// Regex for reverse italicization at the beginning and end of titles.
    ///
    /// Gets rid of any starting <i> or ending </i>.
    pub static ref ITALICS1: Regex = Regex::new(r"^<i>|</i>$").unwrap();

    /// Regex for reverse italicization at the beginning of words.
    ///
    /// Converts any <i> with preceding whitespace * and that whitespace.
    pub static ref ITALICS2: Regex = Regex::new(r"(?P<w>\s+?)<i>").unwrap();

    /// Regex for reverse italicization at the end of words.
    ///
    /// Converts any </i> with following whitespace to that whitespace and *.
    pub static ref ITALICS3: Regex = Regex::new(r"</i>(?P<w>\s+?)").unwrap();

    /// Regex for reverse italization surround by non-whitespace characters.
    ///
    /// Convert any <i> or </i> surrounded by non-whitespace characters to *.
    pub static ref ITALICS4: Regex = Regex::new(r"<i>|</i>").unwrap();
}

/// Add the volume at the beginning of a book that has one.
pub fn add_book_volume(csl_source: &CSLSource, cite: &mut String) {
    cite.push_str(csl_source.volume.as_ref().unwrap());
    cite.push(' ');
}

/// Add the authors. If it's a book, also bold the authors.
pub fn add_authors(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    let mut author = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_long_author")),
        || build_long_author(csl_source.author.as_ref().unwrap()),
    );

    if source_type == &SourceType::Book {
        author = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
            bold(&author)
        });
    }

    author.push_str(", ");

    cite.push_str(&author);
}

/// Add the short author for hereinafters and short cites.
pub fn add_short_author(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    let mut short_author = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_short_author()")),
        || build_short_author(csl_source.author.as_ref().unwrap()),
    );
    if source_type == &SourceType::Book {
        short_author = bold(&short_author);
    }

    cite.push_str(&short_author);
}

/// Builds the long author (*i.e.*, all authors, full names) for various
/// sources.
///
/// This function takes a collection of [`NameVariable`]s and outputs properly
/// formatted author names. The function first iterates through each
/// [`NameVariable`] to create a new vector of authors. Each name is built in
/// the following order:
///
/// * Given name
/// * Non-dropping particle
/// * Family
/// * Suffix
///
/// There is currently no support for the `dropping-particle`, `comma-suffix`,
/// `static-ordering`, `literal`, and `parse-name` fields from CSL JSON's
/// `name-variable` object.
///
/// The function simply concatenates the existing fields with appropriate
/// spaces. The only unique operation deals with the `suffix` field. If the
/// `suffix` is "Jr." or "Sr.", a comma and space precede it. Any other `suffix`
/// gets only a space.
///
/// The function then iterates through the vector of name strings, adding them
/// together into an appropriate form. If there's only one author, the function
/// simply clones that string. All but the last author are iterated through,
/// with ", " added after each. Then, the last ", " is removed and the last
/// author is added with a preceding " & ".
pub fn build_long_author(name_vector: &[NameVariable]) -> String {
    trace!(slog_scope::logger(), "name_vector: {:?}", name_vector);

    let mut long_author = String::with_capacity(256);
    let num_names = name_vector.len();
    for (number, name) in name_vector.iter().enumerate() {
        if name.given.is_some() {
            long_author.push_str(name.given.as_ref().unwrap().as_str());
            long_author.push(' ');
        }
        if name.non_dropping_particle.is_some() {
            long_author.push_str(name.non_dropping_particle.as_ref().unwrap().as_str());
            long_author.push(' ');
        }
        if name.family.is_some() {
            long_author.push_str(name.family.as_ref().unwrap().as_str());
        }
        if name.suffix.is_some() {
            if name.suffix.as_ref().unwrap() == "Jr." || name.suffix.as_ref().unwrap() == "Sr." {
                long_author.push_str(", ");
            } else {
                long_author.push(' ');
            }
            long_author.push_str(name.suffix.as_ref().unwrap().as_str());
        }

        // Note, subtracting 2 (not 1) from `num_names` is needed because
        // `num_names` is the length of the vector but `number` is an index that
        // starts at 0.
        if num_names > 1 && number < num_names - 2 {
            // Multiple authors, and not yet at the second to last.
            long_author.push_str(", ");
        } else if num_names > 1 && number == num_names - 2 {
            // Multiple authors, and at the second to last.
            long_author.push_str(" & ")
        }
    }

    trace!(slog_scope::logger(), "long_author: {:?}", long_author);

    long_author
}

/// Builds the short author (one or two authors' last names, or one and et al.)
/// for sources.
///
/// This function returns a shortened author for short-form citations. It starts
/// by compiling the last name of the first author (non-dropping particle +
/// family name). If there are two authors, the function then adds an " & " and
/// the short name of the second author. If there are three or more authors, the
/// function simply adds " et al."
pub fn build_short_author(name_vector: &[NameVariable]) -> String {
    trace!(slog_scope::logger(), "name_vector: {:?}", name_vector);

    // Create a new string that will probably have sufficient capacity.
    let mut short_author = String::with_capacity(128);

    // Create the short name for the first (and potentially only) author.
    if name_vector[0].non_dropping_particle.is_some() {
        short_author.push_str(name_vector[0].non_dropping_particle.as_ref().unwrap());
        short_author.push(' ');
    }
    if name_vector[0].family.is_some() {
        short_author.push_str(name_vector[0].family.as_ref().unwrap());
    }

    // Then determine what to do based on the number of other authors. If there
    // are two, push on an ampersand and the short name for the second author.
    // Otherwise use the first author's name and an "et al."
    match name_vector.len() {
        1 => {}
        2 => {
            short_author.push_str(" & ");

            if name_vector[1].non_dropping_particle.is_some() {
                short_author.push_str(name_vector[1].non_dropping_particle.as_ref().unwrap());
                short_author.push(' ');
            }
            if name_vector[1].family.is_some() {
                short_author.push_str(name_vector[1].family.as_ref().unwrap());
            }
        }
        _ => short_author.push_str(" et al."),
    }

    trace!(slog_scope::logger(), "short_author: {:?}", short_author);

    short_author
}

/// Add the title.
///
/// If it's a book, also bold it. If it's a chapter, journal, or manuscript,
/// "reverse italicize" it. If it's a case, italicize `In re` and *ex rel*; otherwise let it be.
///
/// TODO: shorten words in case names?
pub fn add_title(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    if source_type == &SourceType::Book {
        let title = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
            bold(csl_source.title.as_ref().unwrap())
        });
        cite.push_str(&title);
    } else if source_type == &SourceType::Chapter
        || source_type == &SourceType::JournalArticle
        || source_type == &SourceType::Manuscript
    {
        let title = slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
            || reverse_italicize(csl_source.title.as_ref().unwrap()),
        );
        cite.push_str(&title);
    } else if source_type == &SourceType::Case {
        let mut title = csl_source.title.as_ref().unwrap().clone();

        // Italicize any un-italicized `In re`s and `ex rel.`s.
        if title.contains("In re ") {
            title = title.replace("In re ", "*In re* ");
        }
        if title.contains(" ex rel. ") {
            title = title.replace(" ex rel. ", " *ex. rel.* ");
        }

        cite.push_str(&title);
    }
}

/// Add the shortened title (if it exists).
///
/// This adds the shortened title for a source. If there isn't one, it uses the
/// long title and warns the user.
pub fn add_short_title(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    let short_title;
    if csl_source.title_short.is_some() {
        if source_type == &SourceType::Book {
            short_title = bold(csl_source.title_short.as_ref().unwrap());
        } else if source_type == &SourceType::Case {
            short_title = format!("*{}*", &csl_source.title_short.as_ref().unwrap());
        } else {
            short_title = slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
                || reverse_italicize(csl_source.title_short.as_ref().unwrap()),
            )
        }
    } else {
        short_title = if source_type == &SourceType::Book {
            bold(csl_source.title.as_ref().unwrap())
        } else if source_type == &SourceType::Case {
            format!("*{}*", csl_source.title.as_ref().unwrap())
        } else {
            slog_scope::scope(
                &slog_scope::logger().new(o!("fn" => "reverse_italicize()")),
                || reverse_italicize(csl_source.title.as_ref().unwrap()),
            )
        };
        warn!(
            slog_scope::logger(),
            "No short title found for {}; using long title for short cites",
            Color::Blue.paint(&csl_source.id)
        );
        eprintln!(
            "  {} No short title found for {}; using long title for short cites",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(&csl_source.id)
        )
    }

    cite.push_str(&short_title);
}

/// Adds article title italicization markdown to a title with HTML markup.
///
/// This function takes a title with HTML markup for italicization and converts
/// it to article-title-style markdown. It essentially flips the italicization,
/// as article titles should generally be italicized while words normally
/// italicized in a title should be Roman.
///
/// The function uses four static regular expressions: [`ITALICS1`],
/// [`ITALICS2`], [`ITALICS3`], and [`ITALICS4`].
pub fn reverse_italicize(title: &str) -> String {
    trace!(slog_scope::logger(), "title: {}", title);

    let mut new_title = title.to_string();

    // If there's no starting <i> or ending </i>, put * at the beginning and end
    if !new_title.starts_with("<i>") {
        new_title = format!("*{}", new_title);
    };
    if !new_title.ends_with("</i>") {
        new_title.push('*');
    }

    // Get rid of any starting <i> or ending </i>
    new_title = ITALICS1.replace_all(&new_title, "").to_string();

    // Then convert any <i> with preceding whitespace a * and that whitespace
    new_title = ITALICS2.replace_all(&new_title, "*$w").to_string();

    // Then convert any </i> with following whitespace to that whitespace and *
    new_title = ITALICS3.replace_all(&new_title, "$w*").to_string();

    // Finally convert any <i> or </i> surrounded by non-whitespace characters to *
    new_title = ITALICS4.replace_all(&new_title, "*").to_string();

    trace!(slog_scope::logger(), "new_title: {}", new_title);

    new_title
}

/// Add the "in" if it's a book chapter.
pub fn add_in(cite: &mut String) {
    cite.push_str(", *in* ");
}

/// Add the volume to articles, chapters, and manuscripts.
pub fn add_other_volume(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    if source_type == &SourceType::Case
        || source_type == &SourceType::JournalArticle
        || source_type == &SourceType::Manuscript
    {
        cite.push_str(", ");
    }
    cite.push_str(csl_source.volume.as_ref().unwrap());
    cite.push(' ');
}

/// The container name (book, journal name, or reporter) for cases, chapters, and articles.
pub fn add_container_name(
    csl_source: &CSLSource,
    source_type: &SourceType,
    user_journals: &Option<UserJournals>,
    cite: &mut String,
) {
    let container_title;
    if source_type == &SourceType::Chapter {
        container_title =
            slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                bold(csl_source.container_title.as_ref().unwrap())
            });
    } else if source_type == &SourceType::Case {
        container_title = csl_source.container_title.as_ref().unwrap().to_string();
    } else if csl_source.container_title_short.is_some() {
        container_title =
            slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                bold(csl_source.container_title_short.as_ref().unwrap())
            });
    } else {
        let short_journal = slog_scope::scope(
            &slog_scope::logger().new(o!("fn" => "build_short_journal()")),
            || build_short_journal(csl_source.container_title.as_ref().unwrap(), user_journals),
        );
        container_title =
            slog_scope::scope(&slog_scope::logger().new(o!("fn" => "bold()")), || {
                bold(&short_journal)
            });
    }

    cite.push_str(&container_title);
}

/// Returns a (hopefully) short form of a journal name.
///
/// Using the tables in the `journalnames` module (and possible user-provided
/// abbreviations), this function tries to return the shortened form of a
/// journal name.
///
/// If the user supplied a set of journal names and abbreviations, the function
/// looks there first.
///
/// If the user didn't, or the journal name is not found in the user journals,
/// the function then turns to the `journalnames` module. The `journalnames`
/// module contains a table of common, special, or saved journal names, and the
/// function looks for the journal name in that table.
///
/// Finally, if the journal name is not found, the function then tries to build
/// a short name using the `replacement` module's replacement tables in the
/// order specified in Indigo Book T18. The program alerts the user to the
/// attempt at building a short journal name.
pub fn build_short_journal(long_journal: &str, user_journals: &Option<UserJournals>) -> String {
    trace!(slog_scope::logger(), "long_journal: {}", long_journal);
    trace!(slog_scope::logger(), "user_journals: {:?}", user_journals);

    // First look through the user-supplied journal names, if given
    if user_journals.is_some() && user_journals.as_ref().unwrap().contains_key(long_journal) {
        let short_journal = user_journals.as_ref().unwrap()[long_journal].clone();
        trace!(slog_scope::logger(), "short_journal: {}", short_journal);
        short_journal
    } else if journalnames::JOURNALNAMES.contains_key(long_journal) {
        // Then look for the short form in the saved names
        let short_journal = journalnames::JOURNALNAMES[long_journal].to_string();
        trace!(slog_scope::logger(), "short_journal: {}", short_journal);
        short_journal
    } else {
        // Otherwise try and build a short journal title
        let mut journal = long_journal.to_string();

        // First go through the multi-word replacements
        for (k, v) in replacements::MULTIWORD.into_iter() {
            if journal.contains(k) {
                journal = journal.replace(k, v);
            }
        }

        // Then break the journal name into the words and iterate through the
        // words looking for replacements.
        let mut short_journal = String::with_capacity(128);
        let words = journal.split_whitespace();
        for word in words {
            if replacements::INSTITUTIONS.contains_key(word) {
                short_journal.push_str(replacements::INSTITUTIONS[word]);
                short_journal.push(' ');
            } else if replacements::ABBREVIATIONS.contains_key(word) {
                short_journal.push_str(replacements::ABBREVIATIONS[word]);
                short_journal.push(' ');
            } else if replacements::GEOGRAPHY.contains_key(word) {
                short_journal.push_str(replacements::GEOGRAPHY[word]);
                short_journal.push(' ');
            } else if replacements::REMOVALS.contains_key(word) {
            } else {
                short_journal.push_str(word);
                short_journal.push(' ');
            }
        }

        warn!(
            slog_scope::logger(),
            "No short journal name found for {}; using {}",
            Color::Blue.paint(long_journal),
            Color::Blue.paint(&short_journal)
        );
        eprintln!(
            "  {} No short journal name found for {}; using {}",
            Color::Yellow.paint("WARN"),
            Color::Blue.paint(long_journal),
            Color::Blue.paint(&short_journal)
        );

        trace!(slog_scope::logger(), "short_journal: {}", short_journal);

        short_journal.trim().to_string()
    }
}

/// Add the journal-forthcoming parenthetical.
///
/// Used for manuscripts with a note and a date.
pub fn add_forthcoming(csl_source: &CSLSource, cite: &mut String) {
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
pub fn add_first_page(csl_source: &CSLSource, cite: &mut String) {
    cite.push(' ');
    cite.push_str(csl_source.page.as_ref().unwrap());
}

/// Add the ending parenthetical with court, edition, editors, translators, and year.
pub fn add_end_parenthetical(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    cite.push_str(" (");

    // If it's a case, add the authority.
    if source_type == &SourceType::Case
        && csl_source.authority.is_some()
        && csl_source.authority.as_ref().unwrap() != "U.S. Supreme Court"
    {
        cite.push_str(csl_source.authority.as_ref().unwrap());
        cite.push(' ');
    }

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
            || build_long_author(csl_source.editor.as_ref().unwrap()),
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
            || build_long_author(csl_source.translator.as_ref().unwrap()),
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
pub fn add_hereinafter(csl_source: &CSLSource, source_type: &SourceType, cite: &mut String) {
    cite.push_str(" [hereinafter ");
    add_short_author(csl_source, source_type, cite);
    cite.push_str(", ");
    add_short_title(csl_source, source_type, cite);
    cite.push(']');
}

/// Add the url.
pub fn add_url(csl_source: &CSLSource, cite: &mut String) {
    cite.push_str(", ");
    cite.push_str(csl_source.url.as_ref().unwrap());
}

/// Adds markdown bolding to a string.
pub fn bold(in_string: &str) -> String {
    trace!(slog_scope::logger(), "in_string: {}", in_string);

    format!("**{}**", in_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod author_tests {
        use super::*;

        #[test]
        fn one_long_author() {
            let name_one = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let test_vec = vec![name_one];
            let long_author = build_long_author(&test_vec);

            assert_eq!(long_author, "Given Non Dropping Particle Family Suffix");
        }

        #[test]
        fn two_long_authors() {
            let name_one = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let name_two = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let test_vec = vec![name_one, name_two];
            let long_author = build_long_author(&test_vec);

            assert_eq!(
            long_author,
            "Given Non Dropping Particle Family Suffix & Given Non Dropping Particle Family Suffix"
        );
        }

        #[test]
        fn three_long_authors() {
            let name_one = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let name_two = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let name_three = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let test_vec = vec![name_one, name_two, name_three];
            let long_author = build_long_author(&test_vec);

            assert_eq!(
            long_author,
            "Given Non Dropping Particle Family Suffix, Given Non Dropping Particle Family Suffix & Given Non Dropping Particle Family Suffix"
        );
        }

        #[test]
        fn one_short_author() {
            let name_one = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let test_vec = vec![name_one];
            let short_author = build_short_author(&test_vec);

            assert_eq!(short_author, "Non Dropping Particle Family");
        }

        #[test]
        fn two_short_authors() {
            let name_one = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let name_two = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let test_vec = vec![name_one, name_two];
            let short_author = build_short_author(&test_vec);

            assert_eq!(
                short_author,
                "Non Dropping Particle Family & Non Dropping Particle Family"
            );
        }

        #[test]
        fn three_short_authors() {
            let name_one = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let name_two = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let name_three = NameVariable {
                family: Some("Family".to_string()),
                given: Some("Given".to_string()),
                non_dropping_particle: Some("Non Dropping Particle".to_string()),
                suffix: Some("Suffix".to_string()),
            };
            let test_vec = vec![name_one, name_two, name_three];
            let short_author = build_short_author(&test_vec);

            assert_eq!(short_author, "Non Dropping Particle Family et al.");
        }

        #[test]
        fn partial_authors() {
            let name_one = NameVariable {
                family: Some("van Beethoven".to_string()),
                given: Some("Ludwig".to_string()),
                non_dropping_particle: None,
                suffix: None,
            };
            let name_two = NameVariable {
                family: Some("Johnson".to_string()),
                given: Some("Sam".to_string()),
                non_dropping_particle: None,
                suffix: Some("Jr.".to_string()),
            };
            let name_three = NameVariable {
                family: Some("Smith".to_string()),
                given: Some("Jane".to_string()),
                non_dropping_particle: None,
                suffix: None,
            };
            let test_vec = vec![name_one, name_two, name_three];
            let long_author = build_long_author(&test_vec);
            let short_author = build_short_author(&test_vec);

            assert_eq!(
                long_author,
                "Ludwig van Beethoven, Sam Johnson, Jr. & Jane Smith"
            );
            assert_eq!(short_author, "van Beethoven et al.");
        }

        #[test]
        fn particles() {
            let name_one = NameVariable {
                family: Some("Stefan".to_string()),
                given: Some("Lindsay".to_string()),
                non_dropping_particle: Some("de".to_string()),
                suffix: None,
            };
            let test_vec = vec![name_one];
            let long_author = build_long_author(&test_vec);
            let short_author = build_short_author(&test_vec);

            assert_eq!(long_author, "Lindsay de Stefan");
            assert_eq!(short_author, "de Stefan");
        }
    }

    mod italicize_tests {
        use super::*;

        #[test]
        fn nothing_italicized() {
            let in_title = "An Article About Some Stuff";
            let out_title = reverse_italicize(in_title);

            assert_eq!(&out_title, "*An Article About Some Stuff*");
        }

        #[test]
        fn only_italicized() {
            let in_title = "<i>United States v. Johnson</i>";
            let out_title = reverse_italicize(in_title);

            assert_eq!(&out_title, "United States v. Johnson");
        }

        #[test]
        fn start_italicized() {
            let in_title = "<i>United States v. Johnson</i> and Other Stuff";
            let out_title = reverse_italicize(in_title);

            assert_eq!(&out_title, "United States v. Johnson *and Other Stuff*");
        }

        #[test]
        fn end_italicized() {
            let in_title = "Some Stuff and <i>United States v. Johnson</i>";
            let out_title = reverse_italicize(in_title);

            assert_eq!(&out_title, "*Some Stuff and* United States v. Johnson");
        }

        #[test]
        fn middle_italicized() {
            let in_title = "Some Stuff About <i>United States v. Johnson</i> and Other Stuff";
            let out_title = reverse_italicize(in_title);

            assert_eq!(
                &out_title,
                "*Some Stuff About* United States v. Johnson *and Other Stuff*",
            );
        }

        #[test]
        fn start_and_end_italicized() {
            let in_title = "<i>United States v. Johnson</i>, Some Stuff, and <i>Smith v. Jones</i>";
            let out_title = reverse_italicize(in_title);

            assert_eq!(
                &out_title,
                "United States v. Johnson*, Some Stuff, and* Smith v. Jones",
            );
        }

        #[test]
        fn start_middle_and_end_italicized() {
            let in_title = "<i>United States v. Johnson</i>, the Rule in <i>Jane's Case</i>, and <i>Smith v. Jones</i>";
            let out_title = reverse_italicize(in_title);

            assert_eq!(
                &out_title,
                "United States v. Johnson*, the Rule in* Jane's Case*, and* Smith v. Jones",
            );
        }

        #[test]
        fn puncuation() {
            let in_title1 = "The Rule in <i>Rooker</i>-<i>Feldman</i>";
            let out_title1 = reverse_italicize(in_title1);

            let in_title2 = "<i>United States v. Johnson</i>'s Rule About Stuff";
            let out_title2 = reverse_italicize(in_title2);

            assert_eq!(&out_title1, "*The Rule in* Rooker*-*Feldman",);

            assert_eq!(&out_title2, "United States v. Johnson*'s Rule About Stuff*");
        }
    }

    mod journal_name_tests {
        use super::*;

        #[test]
        fn saved_journal() {
            let long_name = "ABA Journal of Labor & Employment Law";
            let short_name = build_short_journal(long_name, &None);

            assert_eq!(&short_name, "A.B.A. J. Lab. & Emp. L.");
        }

        #[test]
        fn unknown_journal_simple() {
            let long_name = "Journal of Appellate Practice and Process";
            let short_name = build_short_journal(long_name, &None);

            assert_eq!(&short_name, "J. App. Prac. & Process");
        }

        #[test]
        fn unknown_journal_multiword() {
            let long_name1 = "New York Journal of Products Liability Law";
            let short_name1 = build_short_journal(long_name1, &None);

            let long_name2 = "Boston College Review of Rhode Island Law";
            let short_name2 = build_short_journal(long_name2, &None);

            assert_eq!(&short_name1, "N.Y. J. Prod. Liab. L.");
            assert_eq!(&short_name2, "B.C. Rev. R.I. L.")
        }

        #[test]
        fn unkowable_journal() {
            let long_name = "Not a Journal Title";
            let short_name = build_short_journal(long_name, &None);

            assert_eq!(&short_name, "Not J. Title");
        }

        #[test]
        fn user_journal() {
            let user_journals = Some(std::collections::HashMap::from([
                (
                    "Journal of Stuff that Won't Abbreviate".to_string(),
                    "J. Stuff Won't Abbrev.".to_string(),
                ),
                (
                    "Journal of More Stuff".to_string(),
                    "J. More Stuff".to_string(),
                ),
                (
                    "Legal Stuff Law Review".to_string(),
                    "Legal Stuff L. Rev.".to_string(),
                ),
            ]));
            let long_name = "Journal of Stuff that Won't Abbreviate";
            let short_name = build_short_journal(long_name, &user_journals);

            assert_eq!(&short_name, "J. Stuff Won't Abbrev.");
        }
    }
}
