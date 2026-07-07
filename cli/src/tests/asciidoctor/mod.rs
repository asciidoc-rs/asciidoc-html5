//! Coverage of Asciidoctor's reference documentation pages under
//! `ref/asciidoctor/`, from the CLI's point of view.
//!
//! Asciidoctor is this crate's compatibility oracle. The introduction and
//! get-started pages are shared spec sources tracked from both workspace
//! crates: this crate verifies the command-line invocations they show, while
//! `asciidoc-html5` verifies the API-level conversion. The sdd tool merges the
//! two crates' coverage of each page.

mod cli;
mod get_started;
mod html_backend;
mod index;
