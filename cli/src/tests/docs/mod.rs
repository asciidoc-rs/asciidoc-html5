//! Coverage of the crate's documentation pages under `docs/`, from the CLI's
//! point of view.
//!
//! The introduction and "Convert Your First File" pages are shared
//! documentation tracked from both workspace crates: this crate verifies the
//! command-line invocations they show, while `asciidoc-html5` verifies the API
//! invocations. The sdd tool merges the two crates' coverage of each page.

mod cli;
mod convert_your_first_file;
mod generate_html;
mod index;
mod localization_support;
