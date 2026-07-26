//! Tests placed under `src/tests` so the workspace's `sdd` spec-coverage tool
//! can discover their spec markers (see `sdd/README.md`).
//!
//! These drive the CLI's own conversion pipeline in process. The end-to-end
//! tests that spawn the compiled `adoc` binary live in `tests/cli.rs`.

mod sdd;

mod asciidoctor;
mod asciidoctor_rb;
mod backend;
mod docs;
mod doctype;
mod input_resolution;
mod output_collision;
mod output_naming;
mod path_helpers;
mod run_errors;
mod section_numbers;
mod warnings;
