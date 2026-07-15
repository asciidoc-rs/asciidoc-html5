//! Coverage of Asciidoctor's `extensions` documentation module under
//! `ref/asciidoctor/docs/modules/extensions/pages/`.
//!
//! These pages document Asciidoctor's Ruby extension API and its individual
//! extension points. This crate implements no extension mechanism — none is
//! planned for 1.0 (see the workspace README) — so every page here states a
//! rule with no counterpart in this crate and is tracked in full as
//! non-normative.

mod block_macro_processor;
mod block_processor;
mod compound_block_processor;
mod docinfo_processor;
mod include_processor;
mod index;
mod inline_macro_processor;
mod logging;
mod postprocessor;
mod preprocessor;
mod register;
mod tree_processor;
