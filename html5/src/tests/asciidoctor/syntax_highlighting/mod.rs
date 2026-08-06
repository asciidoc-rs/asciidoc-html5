//! Coverage of Asciidoctor's *Syntax Highlighting* documentation module
//! (`ref/asciidoctor/docs/modules/syntax-highlighting/`).
//!
//! This crate models the *client-side* syntax highlighters — highlight.js and
//! prettify — which colorize source blocks in the browser as the page loads.
//! For these, the converter passes the source through unchanged and adds the
//! metadata and CDN assets the highlighter needs, which is exactly the behavior
//! the `index.adoc` and `highlightjs.adoc` pages describe and this module
//! verifies against `convert`.
//!
//! The module's *build-time* highlighters (CodeRay, Pygments, Rouge) tokenize
//! source into `<span>` markup during conversion by invoking an external Ruby
//! library, and the custom-adapter page documents a Ruby extension API for
//! plugging in more. Both are settled non-goals for this crate's 1.0: the
//! library depends only on `asciidoc-parser`, exposes no Ruby extension
//! surface, and no in-process highlighter can match the Asciidoctor 2.0.26
//! parity oracle byte for byte. Those four pages are still tracked here for
//! full module coverage, but wholly as non-normative.

mod coderay;
mod custom;
mod highlightjs;
mod index;
mod pygments;
mod rouge;
