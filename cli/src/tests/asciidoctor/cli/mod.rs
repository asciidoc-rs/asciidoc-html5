//! Coverage of Asciidoctor's `cli` documentation module under
//! `ref/asciidoctor/docs/modules/cli/`, from the CLI's point of view.
//!
//! These pages document the `asciidoctor` command line interface. This crate's
//! `adoc` command is a thin, native front end that mirrors the parts of that
//! interface it supports — converting a file, reporting its version, and
//! printing help — so it verifies those invocations here. Ruby-specific details
//! (the runtime-environment banner) and features `adoc` does not provide (the
//! `manpage` and `syntax` help topics) are tracked as non-normative.

mod index;
mod io_piping;
mod man1_asciidoctor;
mod options;
mod output_file;
mod process_multiple_files;
mod set_safe_mode;
