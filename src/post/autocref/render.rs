//! The module contains functionality for rendeing the new xml contents.

use super::parser::Branch;
use slog::{debug, o};
use std::collections::HashMap;

/// The main render function.
pub fn render(
    doc_tree: &[Branch],
    refd_notes: Vec<u32>,
    starting_bookmark: u32,
    fn_tree: &[Branch],
) -> Result<(String, String), String> {
    debug!(slog_scope::logger(), "Beginning rendering...");

    // Render document.xml
    let (doc_output, ref_ids) = match slog_scope::scope(
        &slog_scope::logger().new(o!("fn" => "render_doc()")),
        || render_doc(doc_tree, refd_notes, starting_bookmark),
    ) {
        Ok(t) => t,
        Err(e) => return Err(e),
    };

    // Render footnotes.xml
    let fn_output =
        match slog_scope::scope(&slog_scope::logger().new(o!("fn" => "render_fn()")), || {
            render_fn(fn_tree, ref_ids)
        }) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };

    debug!(slog_scope::logger(), "Rendering finished.");
    Ok((doc_output, fn_output))
}

/// Render the `document.xml` contents.
///
/// This function produces the new `document.xml` contents, with bookmark markup
/// added to footnote references. It also builds a [`HashMap`] in which the keys
/// are footnote numbers and the values are the Word reference ids (*e.g.*,
/// "_Ref000000001"). This [`HashMap`] is later used for the cross-reference
/// markup.
///
/// **The Markup for Bookmarks**
///
/// Word's markup for bookmarks consists of two tags. The opening tag includes
/// an `id` (a string consisting of a number) and a `name` (the reference id for
/// that bookmark). *E.g.*:
///
/// ```text
/// <w:bookmarkStart w:id="1" w:name="_Ref000000001"/>
/// ```
/// The `id` is used again in the closing tag. *E.g.*:
///
/// ```text
/// <w:bookmarkEnd w:id="1"/>
/// ```
///
/// The `name` is used when adding cross references to refer to the particular
/// bookmark.
fn render_doc(
    tree: &[Branch],
    refd_notes: Vec<u32>,
    mut starting_bookmark: u32,
) -> Result<(String, HashMap<u32, String>), String> {
    debug!(slog_scope::logger(), "Beginning document rendering...");

    // This `String` is given a 500kB capacity to minimize re-allocation.
    let mut doc_output = String::with_capacity(512000);

    // This is the collection of reference ids (*e.g.*, "_Ref000000001") for
    // each cross-referenced footnote
    let mut ref_ids: HashMap<u32, String> = HashMap::new();

    for branch in tree {
        match branch {
            Branch::Text(text) => doc_output.push_str(text.contents),
            Branch::FootnoteRef(footnote_ref) => {
                // Determine if this footnote reference is ever referred to. If
                // it is, it needs a bookmark.
                if refd_notes.contains(&footnote_ref.number) {
                    // First create a unique reference id
                    let ref_id = create_ref_id(footnote_ref.number);

                    // Add that reference id to the collection
                    ref_ids.insert(footnote_ref.number, ref_id.clone());

                    // Add the markup
                    doc_output.push_str(&format!(
                        r#"<w:bookmarkStart w:id="{}" w:name="{}"/>"#,
                        starting_bookmark, ref_id
                    ));
                    doc_output.push_str(footnote_ref.contents);
                    doc_output
                        .push_str(&format!(r#"<w:bookmarkEnd w:id="{}"/>"#, starting_bookmark));

                    // Iterate the bookmark id
                    starting_bookmark += 1;
                } else {
                    // If it's not ever referred to, just add what was already
                    // there
                    doc_output.push_str(footnote_ref.contents);
                }
            }
            _ => {}
        }
    }

    debug!(slog_scope::logger(), "Document rendering finished.");
    Ok((doc_output, ref_ids))
}

/// Render the `footnotes.xml` contents.
///
/// This function produces the new `footnotes.xml` contents, with markup added
/// to each cross-reference.
///
/// **The Markup for Cross-References**
///
/// Word's markup for cross-references reqires the reference id of the bookmark
/// to which it refers. And because the cross-reference comes in the middle of a
/// string, the markup for the string before the cross-refernce must be closed
/// off, and that markup must be restarted after the cross reference. *E.g.*:
///
/// ```text
/// </w:t></w:r><w:fldSimple w:instr=" NOTEREF _Ref000000001 "><w:r><w:t>1</w:t></w:r></w:fldSimple><w:r><w:t xml:space="preserve">
/// ```
fn render_fn(tree: &[Branch], ref_ids: HashMap<u32, String>) -> Result<String, String> {
    debug!(slog_scope::logger(), "Beginning footnote rendering...");

    // TODO This should probably be a string with some capacity to avoid
    // reallocations.
    let mut fn_output = String::with_capacity(512000);

    for branch in tree {
        match branch {
            Branch::Text(text) => fn_output.push_str(text.contents),
            Branch::CrossRef(cross_ref) => {
                // Add the cross-reference field markup.
                fn_output.push_str(&format!(
                    r#"</w:t></w:r><w:fldSimple w:instr=" NOTEREF {} "><w:r><w:t>{}</w:t></w:r></w:fldSimple><w:r><w:t xml:space="preserve">"#,
                    ref_ids[&cross_ref.number],
                    cross_ref.number
                ));
            }
            _ => {}
        }
    }

    debug!(slog_scope::logger(), "Footnote rendering finished.");
    Ok(fn_output)
}

/// Create a unique reference id.
///
/// This function creates a unique reference id for a footnote reference. It
/// uses that footnote reference's footnote number to create the id.
fn create_ref_id(number: u32) -> String {
    let number_str = number.to_string();
    let mut ref_id = String::with_capacity(13);
    ref_id.push_str("_Ref");

    // Loop through the necessary zeros
    while ref_id.len() < 13 - number_str.len() {
        ref_id.push('0');
    }

    // Add in the footnote number as the unique reference id
    ref_id.push_str(&number_str);

    ref_id
}
