//! This module contains functionality related to user-journals files.

use ron::de::from_str;
use slog::debug;
use std::collections::HashMap;

#[allow(dead_code)]
pub type UserJournals = HashMap<String, String>;

/// Create the user-journal names list.
#[allow(dead_code)]
pub fn build_user_journals(input: &str) -> Result<UserJournals, String> {
    match from_str(input) {
        Ok(u) => {
            debug!(slog_scope::logger(), "User journal file parsed");
            Ok(u)
        }
        Err(e) => {
            let err_msg = format!("error deserializing the user journal file—{}", e);
            Err(err_msg)
        }
    }
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
"#;
        let output = build_user_journals(ron_string).unwrap();

        assert_eq!(&output["Journal of Stuff"], "J. Stuff");
        assert_eq!(&output["Journal of More Stuff"], "J. More Stuff");
    }
}
