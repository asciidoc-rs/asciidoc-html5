//! Coverage of Asciidoctor's *Syntax Highlighting* documentation module
//! (`ref/asciidoctor/docs/modules/syntax-highlighting/`).
//!
//! This crate models the *client-side* syntax highlighters — highlight.js and
//! prettify — which colorize source blocks in the browser as the page loads.
//! For these, the converter passes the source through unchanged and adds the
//! metadata and CDN assets the highlighter needs, which is exactly the behavior
//! these pages describe and this module verifies against `convert`.
//!
//! The module's *build-time* highlighters (CodeRay, Pygments, Rouge — the
//! `coderay.adoc`, `pygments.adoc`, `rouge.adoc` pages) tokenize source into
//! `<span>` markup during conversion by invoking an external Ruby library. That
//! is a settled non-goal here: the library depends only on `asciidoc-parser`,
//! and no in-process highlighter can match the Asciidoctor 2.0.26 parity oracle
//! byte for byte. Those pages, and the custom-adapter page (`custom.adoc`, a
//! Ruby extension API), are therefore not tracked; the two client-side–relevant
//! pages are.

mod highlightjs;
mod index;
