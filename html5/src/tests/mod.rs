//! Tests that live under `src/tests` so the workspace's `sdd` spec-coverage
//! tool can discover their spec markers (see `sdd/README.md`).
//!
//! Behavioral unit tests for an individual module stay next to that module's
//! code. This tree holds the tests that are tied to a tracked specification or
//! documentation file, plus the shared no-op coverage markers in [`sdd`].

mod sdd;

mod asciidoctor;
mod docs;
