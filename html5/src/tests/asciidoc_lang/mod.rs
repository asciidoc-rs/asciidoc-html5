//! Coverage of the AsciiDoc language description pages under
//! `ref/asciidoc-lang/`.
//!
//! The AsciiDoc language description at <https://docs.asciidoc.org> is the
//! specification this renderer implements. Where a page documents rendering
//! behavior this crate produces, an ordinary test verifies it against the
//! closest available API; the descriptive prose that surrounds each rule is
//! tracked as non-normative. The `sdd` tool measures the coverage of each page.

mod blocks;
mod root;
