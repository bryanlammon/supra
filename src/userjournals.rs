//! This module contains functionality related to user-journals files.

use ron::de::from_str;
use slog::debug;
use std::collections::HashMap;

#[allow(dead_code)]
pub type UserJournals = HashMap<String, String>;

/// Create the user-journal names list.
#[allow(dead_code)]
pub fn build_user_journals(input: String) -> Result<UserJournals, String> {
    match from_str(&input) {
        Ok(u) => {
            debug!(slog_scope::logger(), "User journal file parsed");
            Ok(u)
        }
        Err(e) => {
            let err_msg = format!("error deserializing the CSL JSON file—{}", e);
            Err(err_msg)
        }
    }
}

/// Create a blank user-journals file.
///
/// Creats a blank user-journal sfile that users can then fill in with their own
/// journals.
#[allow(dead_code)]
pub fn new_user_journals_ron() {
    let blank_ron = r#"
// Enter your own journal abbreviations into this document.
// All entries must come between the two curly brackets, which start and end the
// file. Each entry should include two quoted strings, separated by a colon. The
// first string is the full journal title. The second string is the
// abbreviation. Put each journal on a separate line, with commas after every
// line. Below is an example:
//
// {
//  "Journal of Stuff":"J. Stuff",
//  "Journal of More Stuff":"J. More Stuff",
// }
//
// There is also a placeholder example below. Feel free to replace that with
// your own journals.

{
    "Full Journal Name":"Abbreviated Name",
}
"#;

    std::fs::write("blank-user-journals.ron", blank_ron)
        .expect("Unable to write blank user-journals file");
}

#[cfg(test)]
mod tets {
    use super::*;

    #[test]
    fn basic_build() {
        let ron_string = r#"
{
    "Journal of Stuff":"J. Stuff",
    "Journal of More Stuff":"J. More Stuff",
}
"#
        .to_string();
        let output = build_user_journals(ron_string).unwrap();

        assert_eq!(&output["Journal of Stuff"], "J. Stuff");
        assert_eq!(&output["Journal of More Stuff"], "J. More Stuff");
    }
}
