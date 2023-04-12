//! This module contains functions for building the citation tracker.
//!
//! To enable the renderer to output `*Id.*`s, we need to know what the last cited source was and whether it was in a string cite. If it was the same source and not in a string, then an `*Id.*` can be used instead. Otherwise, another form is necessary.

use crate::pre::parser::Branch;
use slog::debug;
use std::collections::HashMap;

pub fn build_cite_tracker<'a>(tree: &'a [Branch]) -> HashMap<&'a str, i32> {
    debug!(slog_scope::logger(), "Beginning cite tracker...");

    todo!()
}
