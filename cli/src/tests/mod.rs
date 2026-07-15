//! Tests placed under `src/tests` so the workspace's `sdd` spec-coverage tool
//! can discover their spec markers (see `sdd/README.md`).
//!
//! These drive the CLI's own conversion pipeline in process. The end-to-end
//! tests that spawn the compiled `adoc` binary live in `tests/cli.rs`.

mod sdd;

mod asciidoctor;
mod docs;
mod input_resolution;
mod path_helpers;
