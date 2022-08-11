mod crossref;
mod csljson;
mod lexer;
mod options;
mod parser;
mod render;
mod sourcemap;
mod userjournals;

use slog::o;

/// The options for processing a file.
pub struct SupraOptions {
    offset: i32,
    user_journals: Option<String>,
    smallcaps: bool,
}

impl SupraOptions {
    pub fn new(offset: i32, user_journals: Option<String>, smallcaps: bool) -> SupraOptions {
        SupraOptions {
            offset,
            user_journals,
            smallcaps,
        }
    }
}

/// The main function.
pub fn supra<'a>(
    md_input: &'a str,
    csl_input: &'a str,
    options: SupraOptions,
) -> Result<String, String> {
    // Deserialize the CSL library file.
    let csl_library = match slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "de_csl_json()")),
        || csljson::build_csl_lib(csl_input),
    ) {
        Ok(l) => l,
        Err(e) => return Err(e),
    };

    // Lex the markdown input
    let tokens = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "lexer()")), || {
        lexer::lexer(md_input)
    }) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // Parse the tokens into the syntax tree
    let tree = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "parser()")), || {
        parser::parser(&tokens, options.offset)
    }) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // If there's a user journals list, deserialize it into the user journals map.
    let user_journals;
    if options.user_journals.is_some() {
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "parser()")), || {
            userjournals::build_user_journals(options.user_journals.unwrap())
        }) {
            Ok(u) => user_journals = Some(u),
            Err(e) => return Err(e),
        }
    } else {
        user_journals = None;
    }

    // Build the source_map
    let mut source_map = slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "build_source_map()")),
        || sourcemap::build_source_map(&tree, csl_library, user_journals),
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
    if options.smallcaps {
        output = slog_scope::scope(&slog_scope::logger().new(o!("fn" => "smallcaps()")), || {
            options::smallcaps(&output)
        });
    }

    Ok(output)
}
