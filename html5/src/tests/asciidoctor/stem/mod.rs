//! Coverage of Asciidoctor's *STEM Processing* documentation module
//! (`ref/asciidoctor/docs/modules/stem/`).
//!
//! AsciiDoc carries science, technology, engineering, and math (STEM)
//! expressions as passthroughs; it is the converter's job to hand each
//! expression to a STEM processing library. This crate models the *client-side*
//! path — **MathJax** — exactly as the HTML backend does: the AsciiMath/LaTeX
//! source is emitted verbatim inside the delimiters MathJax expects, and, when
//! the `stem` attribute is set, the standalone page loads MathJax from the CDN
//! to render those expressions in the browser. That is the behavior the
//! `index.adoc` and `mathjax.adoc` pages describe, and this module verifies it
//! against `convert`.
//!
//! The module's other two integrations are *processor*-loaded and settled
//! non-goals for this crate's 1.0:
//!
//! * **Asciidoctor Mathematical** (`mathematical.adoc`) is a Ruby extension
//!   backed by a native library that rasterizes each expression to an image
//!   during conversion — it replaces STEM nodes with image nodes before the
//!   converter ever sees them. It requires a native dependency this crate does
//!   not take.
//! * **The AsciiMath gem** (`asciimath-gem.adoc`) and the surrounding **DocBook
//!   toolchain** (`docbook.adoc`) translate AsciiMath to MathML for the DocBook
//!   backend — a backend this crate does not target.
//!
//! Those three pages are tracked here for full module coverage, but wholly as
//! non-normative.

mod asciimath_gem;
mod docbook;
mod index;
mod mathematical;
mod mathjax;
