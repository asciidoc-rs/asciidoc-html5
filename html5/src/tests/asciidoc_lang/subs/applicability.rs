//! Shared helpers for verifying the "Default … substitution" applicability
//! tables on the `subs` step-description pages.
//!
//! Each such table lists block/inline contexts and whether a given substitution
//! step is applied to them by default. To verify a row we wrap a small probe
//! (whose rendered form differs depending on whether the step ran) in the
//! relevant context and check the output for a `marker` substring that appears
//! only when the step is applied.
//!
//! Probe strings contain `{…}` (attribute references, curved-quote marks), so
//! these helpers concatenate rather than `format!` to avoid brace handling.

use crate::{convert, convert_with, Options};

/// Renders `setup` + `wrapped` and reports whether `marker` (the visible sign
/// the substitution ran) appears.
pub(super) fn ran(setup: &str, wrapped: &str, marker: &str) -> bool {
    let mut src = String::from(setup);
    src.push_str(wrapped);
    convert(&src).contains(marker)
}

/// Renders a standalone document whose author line begins with `author_first`,
/// and reports whether `marker` appears in the rendered byline — i.e. whether
/// the step was applied to the header (author/revision) metadata. The check is
/// scoped to the author `<span>` so markers that also occur in the document
/// shell (e.g. `href` in stylesheet links) can't produce a false positive.
pub(super) fn ran_in_header(setup: &str, author_first: &str, marker: &str) -> bool {
    let mut src = String::from(setup);
    src.push_str("= The Title\n");
    src.push_str(author_first);
    src.push_str(" Blogs\n\nbody\n");

    let html = convert_with(&src, &Options::new().standalone(true));
    let Some(start) = html.find("class=\"author\"") else {
        return false;
    };

    let span = &html[start..];
    let end = span.find("</span>").map_or(span.len(), |i| i);
    span[..end].contains(marker)
}
