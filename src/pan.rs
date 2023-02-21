//! Supra's call to Pandoc.

use slog::debug;
use std::path::Path;

/// Run Pandoc on the pre-processor output.
pub fn pan(
    input: &str,
    output: &Path,
    pandoc_reference: Option<&Path>,
) -> Result<pandoc::PandocOutput, pandoc::PandocError> {
    debug!(slog_scope::logger(), "Starting Pandoc...");

    let mut pandoc = pandoc::new();

    pandoc.set_input(pandoc::InputKind::Pipe(input.to_string()));

    pandoc.set_input_format(
        pandoc::InputFormat::Markdown,
        vec![pandoc::MarkdownExtension::ImplicitHeaderReferences],
    );

    if let Some(r) = pandoc_reference {
        pandoc.add_option(pandoc::PandocOption::ReferenceDoc(r.to_path_buf()));
    }

    pandoc.set_output(pandoc::OutputKind::File(output.to_path_buf()));

    debug!(slog_scope::logger(), "Pandoc complete.");
    pandoc.execute()
}
