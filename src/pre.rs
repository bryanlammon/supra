//! The Supra pre-processor.

mod crossref;
mod csljson;
mod lexer;
mod options;
mod parser;
mod render;
mod sourcemap;
pub mod userjournals;

use slog::{debug, o};
use userjournals::build_user_journals;

pub fn pre<'a>(
    input: &'a str,
    library: &'a str,
    user_journals: &Option<String>,
    offset: i32,
    smallcaps: bool,
) -> Result<String, String> {
    debug!(slog_scope::logger(), "Starting pre-processor...");

    // Deserialize the CSL library file.
    let csl_library = match slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_csl_lib()")),
        || csljson::build_csl_lib(library),
    ) {
        Ok(l) => l,
        Err(e) => return Err(e),
    };

    // Lex the markdown input
    let tokens = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "lexer()")), || {
        lexer::lexer(input)
    }) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // Parse the tokens into the syntax tree
    let tree = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "parser()")), || {
        parser::parser(&tokens, offset)
    }) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // If there's a user journals list, deserialize it into the user journals map.
    let user_journals = match user_journals {
        Some(s) => {
            match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "parser()")), || {
                build_user_journals(s)
            }) {
                Ok(u) => Some(u),
                Err(e) => return Err(e),
            }
        }
        None => None,
    };

    // Build the source_map
    let mut source_map = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_source_map()")),
        || sourcemap::build_source_map(&tree, &csl_library, &user_journals),
    );

    // Create the crossref_map
    let crossref_map = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_crossref_map()")),
        || crossref::build_crossref_map(&tree),
    );

    // Render the output
    let mut output = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "render()")), || {
        render::render(&tree, &mut source_map, &crossref_map)
    });

    // Optionally add True Small Caps
    if smallcaps {
        output = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "smallcaps()")), || {
            options::smallcaps(&output)
        });
    }

    debug!(slog_scope::logger(), "Pre-processing completed.");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_cites() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::FIRSTCITES).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::FIRSTCITESTARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn sources() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::SUPRASOURCES).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::SUPRASOURCESTARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn cases() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::CASES).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::CASESTARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn ids() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::IDCITES).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::IDCITESTARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn puncutation() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::PUNCTUATION).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::PUNCTUATIONTARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn signals() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::SIGNALS).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::SIGNALSTARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn string_cite_test() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::STRINGCITE).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::STRINGCITETARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn cross_refs_test() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer(test_inputs::CROSSREFS).unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let render = render::render(&parsed, &mut source_map, &crossref_map);

        let render_lines = render.lines();
        let target_lines: Vec<&str> = test_inputs::CROSSREFSTARGET.lines().collect();

        for (i, line) in render_lines.enumerate() {
            assert_eq!(line, target_lines[i])
        }
    }

    #[test]
    fn cross_refs_error_test() {
        let library = csljson::build_csl_lib(test_inputs::TESTJSON).unwrap();
        let lexed = lexer::lexer("This sentence's footnote contains an invalid cross-reference.^[*See* *infra* note [?non_existent] and accompanying text.]").unwrap();
        let parsed = parser::parser(&lexed, 0).unwrap();
        let crossref_map = crossref::build_crossref_map(&parsed);
        let mut source_map = sourcemap::build_source_map(&parsed, &library, &None);
        let _ = render::render(&parsed, &mut source_map, &crossref_map);
    }

    mod test_inputs {
        pub const TESTJSON: &str = r#"[{"id": "authorAnotherBookChapter2021","author": [{"family": "Author","given": "Chapter"}],"citation-key": "authorAnotherBookChapter2021","container-title": "The Title of the Chapter Book","edition": "5th","editor": [{"family": "Editor","given": "Book"}],"issued": {"date-parts": [[2021]]},"page": "101","title": "Another Book Chapter Title: The Chapter of a Book","title-short": "Another Book Chapter Title","type": "chapter","volume": "15"},{"id": "authorAnotherBookTitle2021","author": [{"family": "Author","given": "Book"}],"citation-key": "authorAnotherBookTitle2021","issued": {"date-parts": [[2021]]},"title": "Another Book Title: A Title for the Dummy Book","title-short": "Another Book Title","type": "book"},{"id": "authorAnotherJournalArticle2021","author": [{"family": "Author","given": "Article"}],"citation-key": "authorAnotherJournalArticle2021","container-title": "Journal of Good Journal Articles","container-title-short": "J. Good J. Articles","issued": {"date-parts": [[2021]]},"page": "1","title": "Another Journal Article Title: A Journal Article","title-short": "Another Journal Article","type": "article-journal","volume": "1"},{"id": "authorAnotherNotForthcoming2021","author": [{"family": "Author","given": "Manuscipt"}],"citation-key": "authorAnotherNotForthcoming2021","issued": {"date-parts": [[2021]]},"title": "Another Not Yet Forthcoming Manuscript: This Manuscript Is Not Yet Placed","title-short": "Another Not Yet","type": "manuscript"},{"id": "authorBookChapterTitle2021","author": [{"family": "Author","given": "Chapter"}],"citation-key": "authorBookChapterTitle2021","container-title": "The Title of the Chapter Book","edition": "5th","editor": [{"family": "Editor","given": "Book"}],"issued": {"date-parts": [[2021]]},"page": "101","title": "Book Chapter Title: The Chapter of a Book","title-short": "Book Chapter Title","type": "chapter","volume": "15"},{"id": "authorBookTitleTitle2021","author": [{"family": "Author","given": "Book"}],"citation-key": "authorBookTitleTitle2021","edition": "4th","issued": {"date-parts": [[2021]]},"title": "Book Title: A Title for the Dummy Book","title-short": "Book Title","type": "book"},{"id": "authorJournalArticleTitle2021","author": [{"family": "Author","given": "Article"}],"citation-key": "authorJournalArticleTitle2021","container-title": "Journal of Journal Articles","container-title-short": "J. J. Articles","issued": {"date-parts": [[2021]]},"page": "1000","title": "Journal Article Title: A Journal Article","title-short": "Journal Article","type": "article-journal","volume": "99"},{"id": "authorNotForthcomingManuscript2021","author": [{"family": "Author","given": "Manuscipt"}],"citation-key": "authorNotForthcomingManuscript2021","issued": {"date-parts": [[2021]]},"title": "Not Yet Forthcoming Manuscript: This Manuscript Is Not Yet Placed","title-short": "Not Yet","type": "manuscript"},{"id": "cauthorJournalArticleYear2021","author": [{"family": "Cauthor","given": "Article"}],"citation-key": "cauthorJournalArticleYear2021","container-title": "The Other Journal of Journal Articles","issued": {"date-parts": [[2021]]},"page": "501","title": "Journal Article With a Year Volume: This Journal Uses Years as Volumes","title-short": "Year Volume","type": "article-journal","volume": "2021"},{"id": "dauthorTwoAuthorJournalArticle2021","author": [{"family": "Dauthor","given": "Article","suffix": "Jr."},{"family": "Fauthor","given": "Article III"}],"citation-key": "dauthorTwoAuthorJournalArticle2021","container-title": "Journal of Journal Articles","issued": {"date-parts": [[2021]]},"page": "101","title": "Two-Author Journal Article: This Article Has Two Authors","title-short": "Two-Author","type": "article-journal","volume": "51"},{"id": "DoeState2022","authority": "7th Cir.","citation-key": "DoeState2022","container-title": "F.4th","issued": {"date-parts": [[2022,7,25]]},"number": "21-12345","page": "456","title": "Doe v. State","title-short": "Doe","type": "legal_case","URL": "www.fakeurl.edu","volume": "123"},{"id": "gauthorThreeAuthorJournalArticle2021","author": [{"family": "Gauthor","given": "Article","suffix": "Sr."},{"family": "Hauthor","given": "Article","suffix": "Jr."},{"family": "Jauthor","given": "Article III"}],"citation-key": "gauthorThreeAuthorJournalArticle2021","container-title": "Journal of Journal Articles","issued": {"date-parts": [[2021]]},"page": "201","title": "Three-Author Journal Article: This Article Has Three Authors","title-short": "Three-Authors","type": "article-journal","volume": "50"},{"id": "JohnsonSmith2023","authority": "U.S. Supreme Court","citation-key": "JohnsonSmith2023","container-title": "U.S.","issued": {"date-parts": [[2023]]},"page": "456","title": "Johnson v. Smith","title-short": "Johnson","type": "legal_case","volume": "123"},{"id": "kauthorManuscriptTitleNot2021","author": [{"family": "Kauthor","given": "Manuscript"}],"citation-key": "kauthorManuscriptTitleNot2021","container-title": "University of Manuscripts Law Review","issued": {"date-parts": [[2021]]},"title": "Manuscript Title: Not Yet a Journal Article","title-short": "Manuscript Title","type": "manuscript","URL": "www.manuscripts.manuscript/manuscript","volume": "99"},{"id": "lauthorBookTranslatorThis2021","author": [{"family": "Lauthor","given": "Book"}],"citation-key": "lauthorBookTranslatorThis2021","issued": {"date-parts": [[2021]]},"title": "Book With a Translator: This Book Has a Translator","title-short": "Book With a Translator","translator": [{"family": "Translator","given": "Book"}],"type": "book"},{"id": "mauthorBookEditorThis2021","author": [{"family": "Mauthor","given": "Book"}],"citation-key": "mauthorBookEditorThis2021","edition": "2d","editor": [{"family": "Editor","given": "Book"}],"issued": {"date-parts": [[2021]]},"title": "Book With an Editor: This Book Has an Editor","title-short": "Book With an Editor","type": "book"},{"id": "nauthorMultiVolumeBookThis2021","author": [{"family": "Nauthor","given": "Book"}],"citation-key": "nauthorMultiVolumeBookThis2021","issued": {"date-parts": [[2021]]},"title": "Multi-Volume Book: This Book Is One of Several Volumes","title-short": "Multi-Volume Book","type": "book","volume": "10"},{"id": "PlaintiffDefendant1991","authority": "1st Cir.","citation-key": "PlaintiffDefendant1991","container-title": "F.3d","issued": {"date-parts": [[1991]]},"page": "1","title": "Plaintiff A v. Defendant A","title-short": "Plaintiff A","type": "legal_case","volume": "100"},{"id": "PlaintiffDefendant1992","authority": "2d Cir.","citation-key": "PlaintiffDefendant1992","container-title": "F.3d","issued": {"date-parts": [[1992]]},"page": "200","title": "Plaintiff B v. Defendant B","title-short": "Plaintiff B","type": "legal_case","volume": "2"},{"id": "PlaintiffDefendant1993","authority": "3d Cir.","citation-key": "PlaintiffDefendant1993","container-title": "F.3d","issued": {"date-parts": [[1993]]},"page": "33","title": "Plaintiff C v. Defendant C","title-short": "Plaintiff C","type": "legal_case","volume": "333"},{"id": "PlaintiffDefendant1994","authority": "4th Cir.","citation-key": "PlaintiffDefendant1994","container-title": "F.3d","issued": {"date-parts": [[1994]]},"page": "444","title": "Plaintiff D v. Defendant D","title-short": "Plaintiff D","type": "legal_case","volume": "44"},{"id": "PlaintiffDefendant1995","authority": "5th Cir.","citation-key": "PlaintiffDefendant1995","container-title": "F.3d","issued": {"date-parts": [[1995]]},"page": "555","title": "Plaintiff E v. Defendant E","title-short": "Plaintiff E","type": "legal_case","volume": "5"},{"id": "PlaintiffDefendant1996","authority": "6th Cir.","citation-key": "PlaintiffDefendant1996","container-title": "F.3d","issued": {"date-parts": [[1996]]},"page": "6","title": "Plaintiff F v. Defendant F","title-short": "Plaintiff F","type": "legal_case","volume": "600"},{"id": "PlaintiffDefendant1997","authority": "7th Cir.","citation-key": "PlaintiffDefendant1997","container-title": "F.3d","issued": {"date-parts": [[1997]]},"page": "77","title": "Plaintiff G v. Defendant G","title-short": "Plaintiff G","type": "legal_case","volume": "77"},{"id": "PlaintiffDefendant1998","authority": "8th Cir.","citation-key": "PlaintiffDefendant1998","container-title": "F.3d","issued": {"date-parts": [[1998]]},"page": "8","title": "Plaintiff H v. Defendant H","title-short": "Plaintiff H","type": "legal_case","volume": "888"},{"id": "PlaintiffDefendant1999","authority": "9th Cir.","citation-key": "PlaintiffDefendant1999","container-title": "F.3d","issued": {"date-parts": [[1999]]},"page": "9","title": "Plaintiff I v. Defendant I","title-short": "Plaintiff I","type": "legal_case","volume": "9"},{"id": "PlaintiffDefendant2000","authority": "10th Cir.","citation-key": "PlaintiffDefendant2000","container-title": "F.3d","issued": {"date-parts": [[2000]]},"page": "1000","title": "Plaintiff J v. Defendant J","title-short": "Plaintiff J","type": "legal_case","volume": "10"},{"id": "PlaintiffDefendant2001","authority": "11th Cir.","citation-key": "PlaintiffDefendant2001","container-title": "F.3d","issued": {"date-parts": [[2001]]},"page": "1111","title": "Plaintiff K v. Defendant K","title-short": "Plaintiff K","type": "legal_case","volume": "111"},{"id": "PlaintiffDefendant2020","authority": "D.C. Cir.","citation-key": "PlaintiffDefendant2020","container-title": "F.4th","issued": {"date-parts": [[2020]]},"page": "434","title": "Plaintiff L v. Defendant L","title-short": "Plaintiff L","type": "legal_case","volume": "43"},{"id": "PlaintiffDefendant2023","authority": "Fed. Cir.","citation-key": "PlaintiffDefendant2023","container-title": "F.4th","issued": {"date-parts": [[2023]]},"page": "345","title": "Plaintiff M v. Defendant M","title-short": "Plaintiff M","type": "legal_case","volume": "543"},{"id": "UnitedStatesJones2022","authority": "1st Cir.","citation-key": "UnitedStatesJones2022","container-title": "F.4th","issued": {"date-parts": [[2022]]},"page": "5309","title": "United States v. Jones","title-short": "Jones","type": "legal_case","volume": "867"}]"#;

        pub const FIRSTCITES: &str = r#######"
# First Citations

## w/o Pincites

This sentence has a footnote with a book.^[[@authorBookTitleTitle2021].]
This sentence has a footnote with a chapter.^[[@authorBookChapterTitle2021].]
This sentence has a footnote with a journal article.^[[@authorJournalArticleTitle2021].]
This sentence has a footnote with a manuscript.^[[@kauthorManuscriptTitleNot2021].]

## w/ Pincites

This sentence has a footnote with a book with a pincite (and a translator).^[[@lauthorBookTranslatorThis2021] at 10.]
This sentence has a footnote with a chapter with a pincite.^[[@authorAnotherBookChapter2021] at 101.]
This sentence has a footnote with a journal article with a pincite (and a year for a volume).^[[@cauthorJournalArticleYear2021] at 501.]
This sentence has a footnote with a munscript and a pincite (and the manuscript is not forthcoming).^[[@authorNotForthcomingManuscript2021] at 10.]

## More!

This sentence has a footnote with a book with an editor.^[[@mauthorBookEditorThis2021].]
This sentence has a footnote with a multi-volume book.^[[@nauthorMultiVolumeBookThis2021].]
This sentence has a footnote with a journal article with two authors.^[[@dauthorTwoAuthorJournalArticle2021].]
This sentence has a footnote with a journal article with three authors.^[[@gauthorThreeAuthorJournalArticle2021].]
"#######;

        pub const FIRSTCITESTARGET: &str = r#######"
# First Citations

## w/o Pincites

This sentence has a footnote with a book.^[**Book Author**, **Book Title: A Title for the Dummy Book** (4th ed. 2021) [hereinafter **Author**, **Book Title**].]
This sentence has a footnote with a chapter.^[Chapter Author, *Book Chapter Title: The Chapter of a Book*, *in* 15 **The Title of the Chapter Book** 101 (5th ed., Book Editor ed., 2021) [hereinafter Author, *Book Chapter Title*].]
This sentence has a footnote with a journal article.^[Article Author, *Journal Article Title: A Journal Article*, 99 **J. J. Articles** 1000 (2021) [hereinafter Author, *Journal Article*].]
This sentence has a footnote with a manuscript.^[Manuscript Kauthor, *Manuscript Title: Not Yet a Journal Article*, 99 **U. Manuscripts L. Rev.** (forthcoming 2021), www.manuscripts.manuscript/manuscript.]

## w/ Pincites

This sentence has a footnote with a book with a pincite (and a translator).^[**Book Lauthor**, **Book With a Translator: This Book Has a Translator** 10 (Book Translator trans., 2021).]
This sentence has a footnote with a chapter with a pincite.^[Chapter Author, *Another Book Chapter Title: The Chapter of a Book*, *in* 15 **The Title of the Chapter Book** 101, 101 (5th ed., Book Editor ed., 2021) [hereinafter Author, *Another Book Chapter Title*].]
This sentence has a footnote with a journal article with a pincite (and a year for a volume).^[Article Cauthor, *Journal Article With a Year Volume: This Journal Uses Years as Volumes*, 2021 **The Other J. J. Articles** 501, 501.]
This sentence has a footnote with a munscript and a pincite (and the manuscript is not forthcoming).^[Manuscipt Author, *Not Yet Forthcoming Manuscript: This Manuscript Is Not Yet Placed* (forthcoming 2021) (manuscript at 10) [hereinafter Author, *Not Yet*].]

## More!

This sentence has a footnote with a book with an editor.^[**Book Mauthor**, **Book With an Editor: This Book Has an Editor** (2d ed., Book Editor ed., 2021).]
This sentence has a footnote with a multi-volume book.^[10 **Book Nauthor**, **Multi-Volume Book: This Book Is One of Several Volumes** (2021).]
This sentence has a footnote with a journal article with two authors.^[Article Dauthor, Jr. & Article III Fauthor, *Two-Author Journal Article: This Article Has Two Authors*, 51 **J. J. Articles** 101 (2021).]
This sentence has a footnote with a journal article with three authors.^[Article Gauthor, Sr., Article Hauthor, Jr. & Article III Jauthor, *Three-Author Journal Article: This Article Has Three Authors*, 50 **J. J. Articles** 201 (2021).]
"#######;

        /// Note, this includes the contents of `FIRSTCITES` so that supra forms will be used.
        pub const SUPRASOURCES: &str = r#######"
# First Citations

## w/o Pincites

This sentence has a footnote with a book.^[[@authorBookTitleTitle2021].]
This sentence has a footnote with a chapter.^[[@authorBookChapterTitle2021].]
This sentence has a footnote with a journal article.^[[@authorJournalArticleTitle2021].]
This sentence has a footnote with a manuscript.^[[@kauthorManuscriptTitleNot2021].]

## w/ Pincites

This sentence has a footnote with a book with a pincite (and a translator).^[[@lauthorBookTranslatorThis2021] at 10.]
This sentence has a footnote with a chapter with a pincite.^[[@authorAnotherBookChapter2021] at 101.]
This sentence has a footnote with a journal article with a pincite (and a year for a volume).^[[@cauthorJournalArticleYear2021] at 501.]
This sentence has a footnote with a munscript and a pincite (and the manuscript is not forthcoming).^[[@authorNotForthcomingManuscript2021] at 10.]

## More!

This sentence has a footnote with a book with an editor.^[[@mauthorBookEditorThis2021].]
This sentence has a footnote with a multi-volume book.^[[@nauthorMultiVolumeBookThis2021].]
This sentence has a footnote with a journal article with two authors.^[[@dauthorTwoAuthorJournalArticle2021].]
This sentence has a footnote with a journal article with three authors.^[[@gauthorThreeAuthorJournalArticle2021].]

# Supras

This sentence cites to the book in footnote one (with a pincite).^[[@authorBookTitleTitle2021] at 100.]
This sentence cites to the chapter in footnote two (note pincite).^[[@authorBookChapterTitle2021] at 100.]
This sentence cites to the journal article in footnote three (with a pincite).^[[@authorJournalArticleTitle2021] at 100.]
This sentence cites to the manuscript in footnote four (no pincite).^[[@kauthorManuscriptTitleNot2021].]

This sentence cites to the book in footnote five (with a pincite).^[[@lauthorBookTranslatorThis2021] at 100.]
This sentence cites to the chapter in footnote six.^[TODO.]
This sentence cites to the journal article in footnote seven (with a pincite).^[[@cauthorJournalArticleYear2021] at 502.]
This sentence cites to the manuscript in footnote eight (no pincite).^[[@authorNotForthcomingManuscript2021] at 11.]

This sentence cites to the book in footnote nine (with a pincite).^[[@mauthorBookEditorThis2021] at 501.]
This sentence cites to the multi-volume book in footnote ten (no pincite).^[[@nauthorMultiVolumeBookThis2021].]
This sentence cites to the journal article in footnote eleven (with a pincite).^[[@dauthorTwoAuthorJournalArticle2021] at 101.]
This sentence cites to the journal article in footnote twelve (no pincite).^[[@gauthorThreeAuthorJournalArticle2021].]
"#######;

        pub const SUPRASOURCESTARGET: &str = r#######"
# First Citations

## w/o Pincites

This sentence has a footnote with a book.^[**Book Author**, **Book Title: A Title for the Dummy Book** (4th ed. 2021) [hereinafter **Author**, **Book Title**].]
This sentence has a footnote with a chapter.^[Chapter Author, *Book Chapter Title: The Chapter of a Book*, *in* 15 **The Title of the Chapter Book** 101 (5th ed., Book Editor ed., 2021) [hereinafter Author, *Book Chapter Title*].]
This sentence has a footnote with a journal article.^[Article Author, *Journal Article Title: A Journal Article*, 99 **J. J. Articles** 1000 (2021) [hereinafter Author, *Journal Article*].]
This sentence has a footnote with a manuscript.^[Manuscript Kauthor, *Manuscript Title: Not Yet a Journal Article*, 99 **U. Manuscripts L. Rev.** (forthcoming 2021), www.manuscripts.manuscript/manuscript.]

## w/ Pincites

This sentence has a footnote with a book with a pincite (and a translator).^[**Book Lauthor**, **Book With a Translator: This Book Has a Translator** 10 (Book Translator trans., 2021).]
This sentence has a footnote with a chapter with a pincite.^[Chapter Author, *Another Book Chapter Title: The Chapter of a Book*, *in* 15 **The Title of the Chapter Book** 101, 101 (5th ed., Book Editor ed., 2021) [hereinafter Author, *Another Book Chapter Title*].]
This sentence has a footnote with a journal article with a pincite (and a year for a volume).^[Article Cauthor, *Journal Article With a Year Volume: This Journal Uses Years as Volumes*, 2021 **The Other J. J. Articles** 501, 501.]
This sentence has a footnote with a munscript and a pincite (and the manuscript is not forthcoming).^[Manuscipt Author, *Not Yet Forthcoming Manuscript: This Manuscript Is Not Yet Placed* (forthcoming 2021) (manuscript at 10) [hereinafter Author, *Not Yet*].]

## More!

This sentence has a footnote with a book with an editor.^[**Book Mauthor**, **Book With an Editor: This Book Has an Editor** (2d ed., Book Editor ed., 2021).]
This sentence has a footnote with a multi-volume book.^[10 **Book Nauthor**, **Multi-Volume Book: This Book Is One of Several Volumes** (2021).]
This sentence has a footnote with a journal article with two authors.^[Article Dauthor, Jr. & Article III Fauthor, *Two-Author Journal Article: This Article Has Two Authors*, 51 **J. J. Articles** 101 (2021).]
This sentence has a footnote with a journal article with three authors.^[Article Gauthor, Sr., Article Hauthor, Jr. & Article III Jauthor, *Three-Author Journal Article: This Article Has Three Authors*, 50 **J. J. Articles** 201 (2021).]

# Supras

This sentence cites to the book in footnote one (with a pincite).^[**Author**, **Book Title**, *supra* note 1, at 100.]
This sentence cites to the chapter in footnote two (note pincite).^[Author, *Book Chapter Title*, *supra* note 2, at 100.]
This sentence cites to the journal article in footnote three (with a pincite).^[Author, *Journal Article*, *supra* note 3, at 100.]
This sentence cites to the manuscript in footnote four (no pincite).^[Kauthor, *supra* note 4.]

This sentence cites to the book in footnote five (with a pincite).^[**Lauthor**, *supra* note 5, at 100.]
This sentence cites to the chapter in footnote six.^[TODO.]
This sentence cites to the journal article in footnote seven (with a pincite).^[Cauthor, *supra* note 7, at 502.]
This sentence cites to the manuscript in footnote eight (no pincite).^[Author, *Not Yet*, *supra* note 8, at 11.]

This sentence cites to the book in footnote nine (with a pincite).^[**Mauthor**, *supra* note 9, at 501.]
This sentence cites to the multi-volume book in footnote ten (no pincite).^[**Nauthor**, *supra* note 10.]
This sentence cites to the journal article in footnote eleven (with a pincite).^[Dauthor & Fauthor, *supra* note 11, at 101.]
This sentence cites to the journal article in footnote twelve (no pincite).^[Gauthor et al., *supra* note 12.]
"#######;

        pub const CASES: &str = r#######"
# Cases

## Full forms, w/o pincites

This sentence cites to a case.^[[@DoeState2022].]
This sentence cites to two cases.^[*See* [@JohnsonSmith2023]; [@UnitedStatesJones2022].]

## Full forms, w/ pincites

This cites to a case, but it should use the case's short form.^[*But see* [@PlaintiffDefendant1991] at 10.]
This cites to two cases, but it should use their short form.^[*See* [@PlaintiffDefendant1992] at 205; [@PlaintiffDefendant1993] at 33.]

## Short forms, w/o pincites

This sentence cites to a case.^[[@PlaintiffDefendant1994].]
This sentence cites to another case.^[*See* [@PlaintiffDefendant1995].]
This sentence cites a case again, and should be in short form.^[[@PlaintiffDefendant1994].]
This sentence cites a case again, and should be in short form.^[[@PlaintiffDefendant1995].]

## Short forms, w/ pincites

This sentence cites to a case.^[[@PlaintiffDefendant1996] at 10.]
This sentence cites to another case.^[*See* [@PlaintiffDefendant1997] at 85.]
This sentence cites a case again, and should be in short form.^[[@PlaintiffDefendant1996] at 11.]
This sentence cites a case again, and should be in short form.^[[@PlaintiffDefendant1997] at 78.]

## Long & Short Forms

This sentence cites to a case.^[[@PlaintiffDefendant1998]; [@PlaintiffDefendant1999].]
An intervening footnote.^[Nothing of interest.]
An intervening footnote.^[Nothing of interest.]
An intervening footnote.^[Nothing of interest.]
An intervening footnote.^[Nothing of interest.]
This sentence should have a short cite.^[[@PlaintiffDefendant1998] at 10.]
This sentence should have a long cite.^[[@PlaintiffDefendant1999] at 10.]

This sentence should have a long cite.^[[@PlaintiffDefendant1996] at 10.]
This sentence should have a short cite.^[[@PlaintiffDefendant1999] at 10.]
"#######;

        pub const CASESTARGET: &str = r#######"
# Cases

## Full forms, w/o pincites

This sentence cites to a case.^[Doe v. State, 123 F.4th 456 (7th Cir. 2022).]
This sentence cites to two cases.^[*See* Johnson v. Smith, 123 U.S. 456 (2023); United States v. Jones, 867 F.4th 5309 (1st Cir. 2022).]

## Full forms, w/ pincites

This cites to a case, but it should use the case's short form.^[*But see* Plaintiff A v. Defendant A, 100 F.3d 1, 10 (1st Cir. 1991).]
This cites to two cases, but it should use their short form.^[*See* Plaintiff B v. Defendant B, 2 F.3d 200, 205 (2d Cir. 1992); Plaintiff C v. Defendant C, 333 F.3d 33, 33 (3d Cir. 1993).]

## Short forms, w/o pincites

This sentence cites to a case.^[Plaintiff D v. Defendant D, 44 F.3d 444 (4th Cir. 1994).]
This sentence cites to another case.^[*See* Plaintiff E v. Defendant E, 5 F.3d 555 (5th Cir. 1995).]
This sentence cites a case again, and should be in short form.^[*Plaintiff D*, 44 F.3d 444.]
This sentence cites a case again, and should be in short form.^[*Plaintiff E*, 5 F.3d 555.]

## Short forms, w/ pincites

This sentence cites to a case.^[Plaintiff F v. Defendant F, 600 F.3d 6, 10 (6th Cir. 1996).]
This sentence cites to another case.^[*See* Plaintiff G v. Defendant G, 77 F.3d 77, 85 (7th Cir. 1997).]
This sentence cites a case again, and should be in short form.^[*Plaintiff F*, 600 F.3d at 11.]
This sentence cites a case again, and should be in short form.^[*Plaintiff G*, 77 F.3d at 78.]

## Long & Short Forms

This sentence cites to a case.^[Plaintiff H v. Defendant H, 888 F.3d 8 (8th Cir. 1998); Plaintiff I v. Defendant I, 9 F.3d 9 (9th Cir. 1999).]
An intervening footnote.^[Nothing of interest.]
An intervening footnote.^[Nothing of interest.]
An intervening footnote.^[Nothing of interest.]
An intervening footnote.^[Nothing of interest.]
This sentence should have a short cite.^[*Plaintiff H*, 888 F.3d at 10.]
This sentence should have a long cite.^[Plaintiff I v. Defendant I, 9 F.3d 9, 10 (9th Cir. 1999).]

This sentence should have a long cite.^[Plaintiff F v. Defendant F, 600 F.3d 6, 10 (6th Cir. 1996).]
This sentence should have a short cite.^[*Plaintiff I*, 9 F.3d at 10.]
"#######;

        pub const IDCITES: &str = r#######"
# *Id.*s

This sentence cites to one source.^[[@PlaintiffDefendant2000].]
It then cites to it again, which should be an `*Id.*`.^[[@PlaintiffDefendant2000] at 20.]
Then a new source.^[[@PlaintiffDefendant2001] at 12.]
And that source again.^[[@PlaintiffDefendant2001] at 13.]

This sentence cites to the same source twice in one footnote.^[Here's some text. *See* [@PlaintiffDefendant1991] at 10. Here's some more text that is hopefully followed by an `*Id.*`. *See* [@PlaintiffDefendant1991] at 12.]

## Citation Break

This sentence cites to a souce.^[[@PlaintiffDefendant1998].]
This sentence cites to a source not in the library, so it needs a cite breaker.^[[$] A Souce that Isn't in the Library at 12.]
So this sentence should have a short cite, not an `*Id.*`^[[@PlaintiffDefendant1998].]

## *Id.*s for Supra Sources

This sentence cites to a supra source.^[*See* [@cauthorJournalArticleYear2021] at 501.].
This sentence cites to the same source and another one.^[[@cauthorJournalArticleYear2021] at 502; *see also* [@dauthorTwoAuthorJournalArticle2021] at 110.]
This sentence cites to the second author.^[*See* [@dauthorTwoAuthorJournalArticle2021] at 112.]
"#######;

        pub const IDCITESTARGET: &str = r#######"
# *Id.*s

This sentence cites to one source.^[Plaintiff J v. Defendant J, 10 F.3d 1000 (10th Cir. 2000).]
It then cites to it again, which should be an `*Id.*`.^[*Id.* at 20.]
Then a new source.^[Plaintiff K v. Defendant K, 111 F.3d 1111, 12 (11th Cir. 2001).]
And that source again.^[*Id.* at 13.]

This sentence cites to the same source twice in one footnote.^[Here's some text. *See* Plaintiff A v. Defendant A, 100 F.3d 1, 10 (1st Cir. 1991). Here's some more text that is hopefully followed by an `*Id.*`. *See* *id.* at 12.]

## Citation Break

This sentence cites to a souce.^[Plaintiff H v. Defendant H, 888 F.3d 8 (8th Cir. 1998).]
This sentence cites to a source not in the library, so it needs a cite breaker.^[A Souce that Isn't in the Library at 12.]
So this sentence should have a short cite, not an `*Id.*`^[*Plaintiff H*, 888 F.3d 8.]

## *Id.*s for Supra Sources

This sentence cites to a supra source.^[*See* Article Cauthor, *Journal Article With a Year Volume: This Journal Uses Years as Volumes*, 2021 **The Other J. J. Articles** 501, 501.].
This sentence cites to the same source and another one.^[*Id.* at 502; *see also* Article Dauthor, Jr. & Article III Fauthor, *Two-Author Journal Article: This Article Has Two Authors*, 51 **J. J. Articles** 101, 110 (2021).]
This sentence cites to the second author.^[*See* Dauthor & Fauthor, *supra* note 10, at 112.]
"#######;

        pub const PUNCTUATION: &str = r#######"
# Punctuation

Citation sentence.^[*See* [@PlaintiffDefendant2000] at 1001.]
Citation sentence.^[Sentence with a period. [@PlaintiffDefendant2000] at 1002.]
Citation clause with comma.^[Sentence ending in a comma, [@PlaintiffDefendant2000] at 1003.]
Citation clause with a semicolon.^[Sentence with a semicolon; [@PlaintiffDefendant2000] at 1004.]
Citation clause with a colon.^[Sentence with a colon: [@PlaintiffDefendant2000] at 1005.]
Citation sentence with an exclamation point.^[Sentence with an exclamation point! [@PlaintiffDefendant2000] at 1006.]
Citation sentence with a question mark.^[Sentence with a question mark? [@PlaintiffDefendant2000] at 1007.]
"#######;

        pub const PUNCTUATIONTARGET: &str = r#######"
# Punctuation

Citation sentence.^[*See* Plaintiff J v. Defendant J, 10 F.3d 1000, 1001 (10th Cir. 2000).]
Citation sentence.^[Sentence with a period. *Id.* at 1002.]
Citation clause with comma.^[Sentence ending in a comma, *id.* at 1003.]
Citation clause with a semicolon.^[Sentence with a semicolon; *id.* at 1004.]
Citation clause with a colon.^[Sentence with a colon: *id.* at 1005.]
Citation sentence with an exclamation point.^[Sentence with an exclamation point! *Id.* at 1006.]
Citation sentence with a question mark.^[Sentence with a question mark? *Id.* at 1007.]
"#######;

        pub const SIGNALS: &str = r#######"
# Signals

No signal.^[[@PlaintiffDefendant2000] at 1001.]

`*E.g.*` signal.^[*E.g.* [@PlaintiffDefendant2000] at 1002.]
`*e.g.*` signal.^[[@PlaintiffDefendant2000] at 1003; *e.g.* [@PlaintiffDefendant2000] at 1004.]
'e.g.' signal.^[Lead in, e.g. [@PlaintiffDefendant2000] at 1005.]

`*Accord*` signal.^[*Accord* [@PlaintiffDefendant2000] at 1002.]
`*accord*` signal.^[[@PlaintiffDefendant2000] at 1003; *accord* [@PlaintiffDefendant2000] at 1004.]
`accord` signal.^[Lead in, accord [@PlaintiffDefendant2000] at 1005.]

`*See*` signal.^[*See* [@PlaintiffDefendant2000] at 1002.]
`*see*` signal.^[[@PlaintiffDefendant2000] at 1003; *see* [@PlaintiffDefendant2000] at 1004.]
`see` signal.^[Lead in, see [@PlaintiffDefendant2000] at 1005.]

`*See, e.g.*,` signal.^[*See, e.g.*, [@PlaintiffDefendant2000] at 1002.]
`*see, e.g.*,` signal.^[[@PlaintiffDefendant2000] at 1003; *see, e.g.*, [@PlaintiffDefendant2000] at 1004.]
`see, e.g.,` signal.^[Lead in, see, e.g., [@PlaintiffDefendant2000] at 1005.]

`*See also*` signal.^[*See also* [@PlaintiffDefendant2000] at 1002.]
`*see also*` signal.^[[@PlaintiffDefendant2000] at 1003; *see also* [@PlaintiffDefendant2000] at 1004.]
`see also` signal.^[Lead in, see also [@PlaintiffDefendant2000] at 1005.]

`*See also, e.g.*,` signal.^[*See also, e.g.*, [@PlaintiffDefendant2000] at 1002.]
`*see also, e.g.*,` signal.^[[@PlaintiffDefendant2000] at 1003; *see also, e.g.*, [@PlaintiffDefendant2000] at 1004.]
`see also, e.g.,` signal.^[Lead in, see also, e.g., [@PlaintiffDefendant2000] at 1005.]

`*Cf.*` signal.^[*Cf.* [@PlaintiffDefendant2000] at 1002.]
`*cf.*` signal.^[[@PlaintiffDefendant2000] at 1003; *cf.* [@PlaintiffDefendant2000] at 1004.]
`cf.` signal.^[Lead in, cf. [@PlaintiffDefendant2000] at 1005.]

`*Cf., e.g.*,` signal.^[*Cf., e.g.*, [@PlaintiffDefendant2000] at 1002.]
`*cf., e.g.*,` signal.^[[@PlaintiffDefendant2000] at 1003; *cf., e.g.*, [@PlaintiffDefendant2000] at 1004.]
`cf., e.g.,` signal.^[Lead in, cf., e.g., [@PlaintiffDefendant2000] at 1005.]

`*Compare*` signal.^[*Compare* [@PlaintiffDefendant2000] at 1002, *with* [@PlaintiffDefendant2000] at 1002.]
`*compare*` signal.^[[@PlaintiffDefendant2000] at 1003; *compare* [@PlaintiffDefendant2000] at 1004.]
`compare` signal.^[Lead in, compare [@PlaintiffDefendant2000] at 1005.]

`*Contra*` signal.^[*Contra* [@PlaintiffDefendant2000] at 1002.]
`*contra*` signal.^[[@PlaintiffDefendant2000] at 1003; *contra* [@PlaintiffDefendant2000] at 1004.]
`contra` signal.^[Lead in, contra [@PlaintiffDefendant2000] at 1005.]

`*But see*` signal.^[*But see* [@PlaintiffDefendant2000] at 1002.]
`*but see*` signal.^[[@PlaintiffDefendant2000] at 1003; *but see* [@PlaintiffDefendant2000] at 1004.]
`but see` signal.^[Lead in, but see [@PlaintiffDefendant2000] at 1005.]

`*But see, e.g.*,` signal.^[*But see, e.g.*, [@PlaintiffDefendant2000] at 1002.]
`*but see, e.g.*,` signal.^[[@PlaintiffDefendant2000] at 1003; *but see, e.g.*, [@PlaintiffDefendant2000] at 1004.]
`but see` signal.^[Lead in, but see [@PlaintiffDefendant2000] at 1005.]

`*But cf.*` signal.^[*But cf.* [@PlaintiffDefendant2000] at 1002.]
`*but cf.*` signal.^[[@PlaintiffDefendant2000] at 1003; *but cf.* [@PlaintiffDefendant2000] at 1004.]
`but cf.` signal.^[Lead in, but cf. [@PlaintiffDefendant2000] at 1005.]

`*But cf., e.g.*,` signal.^[*But cf., e.g.*, [@PlaintiffDefendant2000] at 1002.]
`*but cf., e.g.*,` signal.^[[@PlaintiffDefendant2000] at 1003; *but cf., e.g.*, [@PlaintiffDefendant2000] at 1004.]
`but cf., e.g.,` signal.^[Lead in, but cf., e.g., [@PlaintiffDefendant2000] at 1005.]

`*See generally*` signal.^[*See generally* [@PlaintiffDefendant2000] at 1002.]
`*see generally*` signal.^[[@PlaintiffDefendant2000] at 1003; *see generally* [@PlaintiffDefendant2000] at 1004.]
`see generally` signal.^[Lead in, see generally [@PlaintiffDefendant2000] at 1005.]

`*See generally, e.g.*,` signal.^[*See generally, e.g.*, [@PlaintiffDefendant2000] at 1002.]
`*see generally, e.g.*,` signal.^[[@PlaintiffDefendant2000] at 1003; *see generally, e.g.*, [@PlaintiffDefendant2000] at 1004.]
`see generally, e.g.,` signal.^[Lead in, see generally, e.g., [@PlaintiffDefendant2000] at 1005.]

Signal without a cite.^[See the discussion in Part II.]

`*See*` signal.^[*See* [@PlaintiffDefendant2000] at 1002.]
"#######;

        pub const SIGNALSTARGET: &str = r#######"
# Signals

No signal.^[Plaintiff J v. Defendant J, 10 F.3d 1000, 1001 (10th Cir. 2000).]

`*E.g.*` signal.^[*E.g.* *id.* at 1002.]
`*e.g.*` signal.^[*Id.* at 1003; *e.g.* *id.* at 1004.]
'e.g.' signal.^[Lead in, e.g. *id.* at 1005.]

`*Accord*` signal.^[*Accord* *id.* at 1002.]
`*accord*` signal.^[*Id.* at 1003; *accord* *id.* at 1004.]
`accord` signal.^[Lead in, accord *id.* at 1005.]

`*See*` signal.^[*See* *id.* at 1002.]
`*see*` signal.^[*Id.* at 1003; *see* *id.* at 1004.]
`see` signal.^[Lead in, see *id.* at 1005.]

`*See, e.g.*,` signal.^[*See, e.g.*, *id.* at 1002.]
`*see, e.g.*,` signal.^[*Id.* at 1003; *see, e.g.*, *id.* at 1004.]
`see, e.g.,` signal.^[Lead in, see, e.g., *id.* at 1005.]

`*See also*` signal.^[*See also* *id.* at 1002.]
`*see also*` signal.^[*Id.* at 1003; *see also* *id.* at 1004.]
`see also` signal.^[Lead in, see also *id.* at 1005.]

`*See also, e.g.*,` signal.^[*See also, e.g.*, *id.* at 1002.]
`*see also, e.g.*,` signal.^[*Id.* at 1003; *see also, e.g.*, *id.* at 1004.]
`see also, e.g.,` signal.^[Lead in, see also, e.g., *id.* at 1005.]

`*Cf.*` signal.^[*Cf.* *id.* at 1002.]
`*cf.*` signal.^[*Id.* at 1003; *cf.* *id.* at 1004.]
`cf.` signal.^[Lead in, cf. *id.* at 1005.]

`*Cf., e.g.*,` signal.^[*Cf., e.g.*, *id.* at 1002.]
`*cf., e.g.*,` signal.^[*Id.* at 1003; *cf., e.g.*, *id.* at 1004.]
`cf., e.g.,` signal.^[Lead in, cf., e.g., *id.* at 1005.]

`*Compare*` signal.^[*Compare* *id.* at 1002, *with* *id.* at 1002.]
`*compare*` signal.^[*Id.* at 1003; *compare* *id.* at 1004.]
`compare` signal.^[Lead in, compare *id.* at 1005.]

`*Contra*` signal.^[*Contra* *id.* at 1002.]
`*contra*` signal.^[*Id.* at 1003; *contra* *id.* at 1004.]
`contra` signal.^[Lead in, contra *id.* at 1005.]

`*But see*` signal.^[*But see* *id.* at 1002.]
`*but see*` signal.^[*Id.* at 1003; *but see* *id.* at 1004.]
`but see` signal.^[Lead in, but see *id.* at 1005.]

`*But see, e.g.*,` signal.^[*But see, e.g.*, *id.* at 1002.]
`*but see, e.g.*,` signal.^[*Id.* at 1003; *but see, e.g.*, *id.* at 1004.]
`but see` signal.^[Lead in, but see *id.* at 1005.]

`*But cf.*` signal.^[*But cf.* *id.* at 1002.]
`*but cf.*` signal.^[*Id.* at 1003; *but cf.* *id.* at 1004.]
`but cf.` signal.^[Lead in, but cf. *id.* at 1005.]

`*But cf., e.g.*,` signal.^[*But cf., e.g.*, *id.* at 1002.]
`*but cf., e.g.*,` signal.^[*Id.* at 1003; *but cf., e.g.*, *id.* at 1004.]
`but cf., e.g.,` signal.^[Lead in, but cf., e.g., *id.* at 1005.]

`*See generally*` signal.^[*See generally* *id.* at 1002.]
`*see generally*` signal.^[*Id.* at 1003; *see generally* *id.* at 1004.]
`see generally` signal.^[Lead in, see generally *id.* at 1005.]

`*See generally, e.g.*,` signal.^[*See generally, e.g.*, *id.* at 1002.]
`*see generally, e.g.*,` signal.^[*Id.* at 1003; *see generally, e.g.*, *id.* at 1004.]
`see generally, e.g.,` signal.^[Lead in, see generally, e.g., *id.* at 1005.]

Signal without a cite.^[See the discussion in Part II.]

`*See*` signal.^[*See* *id.* at 1002.]
"#######;

        pub const STRINGCITE: &str = r#######"
# String Cites

This footnote has a string cite.^[*See, e.g.*, [@PlaintiffDefendant1998] at 12; [@dauthorTwoAuthorJournalArticle2021] at 110.]
This footnote should have a short cite, since the source was previously cited in a string.^[*Cf.* [@dauthorTwoAuthorJournalArticle2021] at 112.]
"#######;

        pub const STRINGCITETARGET: &str = r#######"
# String Cites

This footnote has a string cite.^[*See, e.g.*, Plaintiff H v. Defendant H, 888 F.3d 8, 12 (8th Cir. 1998); Article Dauthor, Jr. & Article III Fauthor, *Two-Author Journal Article: This Article Has Two Authors*, 51 **J. J. Articles** 101, 110 (2021).]
This footnote should have a short cite, since the source was previously cited in a string.^[*Cf.* Dauthor & Fauthor, *supra* note 1, at 112.]
"#######;

        pub const CROSSREFS: &str = r#######"
# Cross References

This sentence's footnote has an ID.^[[?first] Footnote contents.]
This sentence refers back to the first footnote.^[[?second] *See* *supra* note [?first].]
This sentence refers to the next note.^[[?third] *But see* *infra* note [?fourth] and accompanying text.]

This sentence refers back to the first two.^[[?fourth] *E.g.*, text accompanying *supra* notes [?first] & [?second].]
And this sentence refers to the middle three.^[[?fifth] *Contra* *supra* notes [?second]--[?fourth].]
"#######;

        pub const CROSSREFSTARGET: &str = r#######"
# Cross References

This sentence's footnote has an ID.^[Footnote contents.]
This sentence refers back to the first footnote.^[*See* *supra* note 1.]
This sentence refers to the next note.^[*But see* *infra* note 4 and accompanying text.]

This sentence refers back to the first two.^[*E.g.*, text accompanying *supra* notes 1 & 2.]
And this sentence refers to the middle three.^[*Contra* *supra* notes 2--4.]
"#######;
    }
}
