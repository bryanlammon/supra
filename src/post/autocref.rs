//! Module for changing footnote cross-references to fields.

mod bookmarks;
mod lexer;
mod parser;
mod render;

use slog::o;

pub fn autocref(mut doc: String, mut fns: String) -> Result<(String, String), String> {
    // Determine the starting bookmark id number
    let starting_bookmark = match slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "starting_bookmark()")),
        || bookmarks::starting_bookmark(&doc),
    ) {
        Ok(i) => i,
        Err(e) => return Err(e),
    };

    // Lex the inputs
    let (doc_tokens, fn_tokens) =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "lex()")), || {
            lexer::lex(&doc, &fns)
        }) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

    // Parse the tokens
    let (doc_branches, fn_branches, refd_fns) =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "parser()")), || {
            parser::parser(&doc_tokens, &fn_tokens)
        }) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

    // Render the output
    (doc, fns) = match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "render()")), || {
        render::render(&doc_branches, refd_fns, starting_bookmark, &fn_branches)
    }) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    Ok((doc, fns))
}
