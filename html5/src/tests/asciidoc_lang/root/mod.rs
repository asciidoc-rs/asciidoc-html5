//! Coverage of the AsciiDoc language description's `ROOT` module pages — the
//! introductory and reference material at the top of the language docs
//! (`ref/asciidoc-lang/docs/modules/ROOT/pages/`).
//!
//! These pages introduce and motivate the language rather than specifying
//! rendering rules, so they are largely non-normative here. The one page that
//! describes concrete processor behavior this crate delivers —
//! `document-processing.adoc` — is verified against `convert`/`load`.

mod asciidoc_vs_markdown;
mod comments;
mod document_processing;
mod document_structure;
mod faq;
mod glossary;
mod index;
mod key_concepts;
mod normalization;
