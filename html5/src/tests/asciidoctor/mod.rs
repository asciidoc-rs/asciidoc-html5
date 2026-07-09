//! Coverage of the Asciidoctor reference documentation pages under
//! `ref/asciidoctor/`.
//!
//! Asciidoctor's `html5` backend is this renderer's compatibility oracle, so
//! its documentation is a spec source the `sdd` tool measures coverage against.
//! Overview pages carry little that is testable here: where a page shows an
//! invocation with a counterpart in this crate, an ordinary test verifies that
//! closest available API, and the rest — prose that describes rather than
//! specifies — is tracked as non-normative.

mod api;
mod docbook_backend;
mod get_started;
mod html_backend;
mod index;
mod manpage_backend;
mod reference_safe_mode;
mod safe_modes;
mod tooling;
mod whats_new;
