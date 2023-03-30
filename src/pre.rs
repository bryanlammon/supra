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
        &slog_scope::logger().new(o!("fn" => "de_csl_json()")),
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
