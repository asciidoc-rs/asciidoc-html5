//! Coverage of Asciidoctor's reference documentation pages under
//! `ref/asciidoctor/`, from the CLI's point of view.
//!
//! Asciidoctor is this crate's compatibility oracle. The introduction page is a
//! shared spec source tracked from both workspace crates: this crate verifies
//! the command-line invocation it shows, while `asciidoc-html5` verifies the
//! API invocation. The sdd tool merges the two crates' coverage of the page.

mod index;
