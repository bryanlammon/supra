//! The module contains functionality related to creating the CSL libary.

use serde::Deserialize;
use slog::debug;

/// Struct holding metadata for a source in a CSL JSON file.
///
/// This struct holds the data for each source deserialized from a CSL JSON
/// library. Note, this struct holds only the data that Supra currently uses.
/// Any other data is discarded.
///
/// Based on the [JSON schema for CSL data].
///
/// Note, some fields in the CSL JSON schema take multiple types. This is not
/// yet supported. Instead, each field's type is based on the type that Zotero
/// and Better BibTeX output.
///
/// [JSON schema for CSL data]:
/// (https://github.com/citation-style-language/schema/blob/master/schemas/input/csl-data.json).
#[derive(Debug, Deserialize)]
pub struct CSLSource {
    #[serde(rename(deserialize = "type"))]
    pub source_type: Option<String>,
    pub id: String, // The only non-optional field
    pub author: Option<Vec<NameVariable>>,
    pub editor: Option<Vec<NameVariable>>,
    pub translator: Option<Vec<NameVariable>>,
    pub issued: Option<DateVariable>,
    #[serde(rename(deserialize = "container-title"))]
    pub container_title: Option<String>,
    #[serde(rename(deserialize = "container-title-short"))]
    pub container_title_short: Option<String>,
    pub edition: Option<String>,
    pub page: Option<String>,
    pub title: Option<String>,
    #[serde(rename(deserialize = "title-short"))]
    pub title_short: Option<String>,
    #[serde(rename(deserialize = "URL"))]
    pub url: Option<String>,
    pub volume: Option<String>,
}

/// Struct holding the CSL JSON `name-variable` data.
#[derive(Debug, Deserialize)]
pub struct NameVariable {
    pub family: Option<String>,
    pub given: Option<String>,
    #[serde(rename(deserialize = "non-dropping-particle"))]
    pub non_dropping_particle: Option<String>,
    pub suffix: Option<String>,
}

/// Struct holding the CSL JSON `date-variable` data.
///
/// Note, `date-parts` data is a collection of collections of one-to-three
/// integers indicating the year, month, and day.
#[derive(Debug, Deserialize)]
pub struct DateVariable {
    #[serde(rename(deserialize = "date-parts"))]
    pub date_parts: Option<Vec<Vec<u32>>>,
    pub season: Option<u32>,
}

/// Deserialize the CSL JSON library.
///
/// Deserialize a string of JSON into a collection of [`CSLSource`]s.
pub fn build_csl_lib(csl_string: &str) -> Result<Vec<CSLSource>, String> {
    debug!(slog_scope::logger(), "Starting CSL JSON parsing...");
    match serde_json::from_str(csl_string) {
        Ok(r) => {
            debug!(slog_scope::logger(), "CSL JSON parsed");
            Ok(r)
        }
        Err(e) => {
            let err_msg = format!("error deserializing the CSL JSON—{}", e);
            Err(err_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let csl_string = r#"[
                {
                    "id": "authorBookChapterTitle2021",
                    "author": [
                        {
                            "family": "Author",
                            "given": "Book"
                        }
                    ],
                    "container-title": "The Title of the Chapter Book",
                    "edition": "5th",
                    "editor": [
                        {
                            "family": "Editor",
                            "given": "Book"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "page": "101",
                    "title": "Book Chapter Title: The Chapter of a Book",
                    "title-short": "Book Chapter Title",
                    "type": "chapter",
                    "volume": "15"
                },
                {
                    "id": "authorBookTitleTitle2021",
                    "author": [
                        {
                            "family": "Author",
                            "given": "Book"
                        }
                    ],
                    "edition": "4th",
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Book Title: A Title for the Dummy Book",
                    "title-short": "Book Title",
                    "type": "book"
                },
                {
                    "id": "authorJournalArticleTitle2021",
                    "author": [
                        {
                            "family": "Author",
                            "given": "Article"
                        }
                    ],
                    "container-title": "Journal of Journal Articles",
                    "container-title-short": "J. J. Articles",
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "page": "1000",
                    "title": "Journal Article Title: A Journal Article",
                    "title-short": "Journal Article",
                    "type": "article-journal",
                    "volume": "99"
                },
                {
                    "id": "authorNotForthcomingManuscript2021",
                    "author": [
                        {
                            "family": "Author",
                            "given": "Manuscipt"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Not Yet Forthcoming Manuscript: This Manuscript Is Not Yet Placed",
                    "title-short": "Not Yet",
                    "type": "manuscript"
                },
                {
                    "id": "cauthorJournalArticleYear2021",
                    "author": [
                        {
                            "family": "Cauthor",
                            "given": "Article"
                        }
                    ],
                    "container-title": "The Other Journal of Journal Articles",
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "page": "501",
                    "title": "Journal Article With a Year Volume: This Journal Uses Years as Volumes",
                    "title-short": "Year Volume",
                    "type": "article-journal",
                    "volume": "2021"
                },
                {
                    "id": "dauthorTwoAuthorJournalArticle2021",
                    "author": [
                        {
                            "family": "Dauthor",
                            "given": "Article",
                            "suffix": "Jr."
                        },
                        {
                            "family": "Fauthor",
                            "given": "Article III"
                        }
                    ],
                    "container-title": "Journal of Journal Articles",
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "page": "101",
                    "title": "Two-Author Journal Article: This Article Has Two Authors",
                    "title-short": "Two-Author",
                    "type": "article-journal",
                    "volume": "51"
                },
                {
                    "id": "gauthorThreeAuthorJournalArticle2021",
                    "author": [
                        {
                            "family": "Gauthor",
                            "given": "Article",
                            "suffix": "Sr."
                        },
                        {
                            "family": "Hauthor",
                            "given": "Article",
                            "suffix": "Jr."
                        },
                        {
                            "family": "Jauthor",
                            "given": "Article III"
                        }
                    ],
                    "container-title": "Journal of Journal Articles",
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "page": "201",
                    "title": "Three-Author Journal Article: This Article Has Three Authors",
                    "title-short": "Three-Authors",
                    "type": "article-journal",
                    "volume": "50"
                },
                {
                    "id": "kauthorManuscriptTitleNot2021",
                    "author": [
                        {
                            "family": "Kauthor",
                            "given": "Manuscript"
                        }
                    ],
                    "container-title": "University of Manuscripts Law Review",
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Manuscript Title: Not Yet a Journal Article",
                    "title-short": "Manuscript Title",
                    "type": "manuscript",
                    "URL": "www.manuscripts.manuscript/manuscript",
                    "volume": "99"
                },
                {
                    "id": "lauthorBookTranslatorThis2021",
                    "author": [
                        {
                            "family": "Lauthor",
                            "given": "Book"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Book With a Translator: This Book Has a Translator",
                    "title-short": "Book With a Translator",
                    "translator": [
                        {
                            "family": "Translator",
                            "given": "Book"
                        }
                    ],
                    "type": "book"
                },
                {
                    "id": "mauthorBookEditorThis2021",
                    "author": [
                        {
                            "family": "Mauthor",
                            "given": "Book"
                        }
                    ],
                    "edition": "2d",
                    "editor": [
                        {
                            "family": "Editor",
                            "given": "Book"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Book With an Editor: This Book Has an Editor",
                    "title-short": "Book With an Editor",
                    "type": "book"
                },
                {
                    "id": "nauthorMultiVolumeBookThis2021",
                    "author": [
                        {
                            "family": "Nauthor",
                            "given": "Book"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Multi-Volume Book: This Book Is One of Several Volumes",
                    "title-short": "Multi-Volume Book",
                    "type": "book",
                    "volume": "10"
                }
            ]
            "#;
        let deser_csl_result = build_csl_lib(csl_string);
        assert!(deser_csl_result.is_ok());
    }

    #[test]
    /// Ensure that a non-JSON file returns an appropriate error.
    fn non_json() {
        let csl_string = r#"This ain't no JSON library..."#;
        let deser_csl_result = build_csl_lib(csl_string);
        assert!(deser_csl_result
            .unwrap_err()
            .contains("error deserializing the CSL JSON"));
    }

    #[test]
    /// Ensure that a JSON file with an error (*e.g.*, extra comma) returns an
    /// appropriate error.
    fn json_error() {
        // TODO
    }

    #[test]
    /// Ensure an error for no id with an appropriate error message.
    fn no_id() {
        let json_string = r#"[
                {
                    "author": [
                        {
                            "family": "Author",
                            "given": "Book"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Book Title: A Title for the Book",
                    "title-short": "Book Title",
                    "type": "book"
                }
            ]"#;

        let result = build_csl_lib(json_string);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing field `id`"));
    }

    #[test]
    /// Test all fields used for books.
    fn book() {
        let json_string = r#"[
                {
                    "id": "authorBookTitleTitle2021",
                    "author": [
                        {
                            "family": "Author",
                            "given": "Book"
                        }
                    ],
                    "edition": "2d",
                    "editor": [
                        {
                            "family": "Editor",
                            "given": "Book"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "title": "Book Title: A Title for the Book",
                    "title-short": "Book Title",
                    "translator": [
                        {
                            "family": "Translator",
                            "given": "Book"
                        }
                    ],
                    "type": "book",
                    "volume": "15"
                }
            ]"#;

        let result = build_csl_lib(json_string);
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap()[0].id, "authorBookTitleTitle2021");
        assert_eq!(
            result.as_ref().unwrap()[0].author.as_ref().unwrap()[0]
                .family
                .as_ref()
                .unwrap(),
            "Author"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].author.as_ref().unwrap()[0]
                .given
                .as_ref()
                .unwrap(),
            "Book"
        );
        assert_eq!(result.as_ref().unwrap()[0].edition.as_ref().unwrap(), "2d");
        assert_eq!(
            result.as_ref().unwrap()[0].editor.as_ref().unwrap()[0]
                .family
                .as_ref()
                .unwrap(),
            "Editor"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].editor.as_ref().unwrap()[0]
                .given
                .as_ref()
                .unwrap(),
            "Book"
        );
        assert_eq!(
            result.as_ref().unwrap()[0]
                .issued
                .as_ref()
                .unwrap()
                .date_parts
                .as_ref()
                .unwrap()[0][0],
            2021
        );
        assert_eq!(
            result.as_ref().unwrap()[0].title.as_ref().unwrap(),
            "Book Title: A Title for the Book"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].title_short.as_ref().unwrap(),
            "Book Title"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].translator.as_ref().unwrap()[0]
                .family
                .as_ref()
                .unwrap(),
            "Translator"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].translator.as_ref().unwrap()[0]
                .given
                .as_ref()
                .unwrap(),
            "Book"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].source_type.as_ref().unwrap(),
            "book"
        );
        assert_eq!(result.as_ref().unwrap()[0].volume.as_ref().unwrap(), "15");
    }

    #[test]
    /// Test all fields used for chapters.
    fn chapter() {
        let json_string = r#"[
                {
                    "id": "authorBookSectionTitle2021",
                    "author": [
                        {
                            "family": "Author",
                            "given": "Chapter"
                        }
                    ],
                    "container-title": "The Title of the Chaptered Book",
                    "edition": "5th",
                    "editor": [
                        {
                            "family": "Editor",
                            "given": "Book"
                        }
                    ],
                    "issued": {
                        "date-parts": [
                            [
                                2021
                            ]
                        ]
                    },
                    "page": "12",
                    "title": "Book Chapter Title: The Chapter of a Book",
                    "title-short": "Book Chapter Title",
                    "translator": [
                        {
                            "family": "Translator",
                            "given": "Book"
                        }
                    ],
                    "type": "chapter",
                    "volume": "99"
                }
            ]"#;

        let result = build_csl_lib(json_string);
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap()[0].id, "authorBookSectionTitle2021");
        assert_eq!(
            result.as_ref().unwrap()[0].author.as_ref().unwrap()[0]
                .family
                .as_ref()
                .unwrap(),
            "Author"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].author.as_ref().unwrap()[0]
                .given
                .as_ref()
                .unwrap(),
            "Chapter"
        );
        assert_eq!(
            result.as_ref().unwrap()[0]
                .container_title
                .as_ref()
                .unwrap(),
            "The Title of the Chaptered Book"
        );
        assert_eq!(result.as_ref().unwrap()[0].edition.as_ref().unwrap(), "5th");
        assert_eq!(
            result.as_ref().unwrap()[0].editor.as_ref().unwrap()[0]
                .family
                .as_ref()
                .unwrap(),
            "Editor"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].editor.as_ref().unwrap()[0]
                .given
                .as_ref()
                .unwrap(),
            "Book"
        );
        assert_eq!(
            result.as_ref().unwrap()[0]
                .issued
                .as_ref()
                .unwrap()
                .date_parts
                .as_ref()
                .unwrap()[0][0],
            2021
        );
        assert_eq!(result.as_ref().unwrap()[0].page.as_ref().unwrap(), "12");
        assert_eq!(
            result.as_ref().unwrap()[0].title.as_ref().unwrap(),
            "Book Chapter Title: The Chapter of a Book"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].title_short.as_ref().unwrap(),
            "Book Chapter Title"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].translator.as_ref().unwrap()[0]
                .family
                .as_ref()
                .unwrap(),
            "Translator"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].translator.as_ref().unwrap()[0]
                .given
                .as_ref()
                .unwrap(),
            "Book"
        );
        assert_eq!(
            result.as_ref().unwrap()[0].source_type.as_ref().unwrap(),
            "chapter"
        );
        assert_eq!(result.as_ref().unwrap()[0].volume.as_ref().unwrap(), "99");
    }

    #[test]
    /// Test all fields used for journal articles
    fn journal_article() {
        // TODO
    }

    #[test]
    /// Test all fields used for manuscripts.
    fn manuscript() {
        // TODO
    }

    #[test]
    /// Test all fields for [`NameVariable`].
    fn names() {
        // TODO
    }

    #[test]
    /// Test all fields for [`DateVariable`].
    fn dates() {
        // TODO
    }

    #[test]
    /// Test multiple authors.
    fn multi_author() {
        // TODO
    }

    #[test]
    /// Test mulitple editors.
    fn multi_editor() {
        // TODO
    }

    #[test]
    /// Test multiple translators.
    fn multi_translator() {
        // TODO
    }
}
